package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

type bundleAssetReference struct {
	WebPath    string
	ConfigPath string
}

func collectBundleAssetReferences(bundle SiteBundle) ([]bundleAssetReference, error) {
	var refs []bundleAssetReference

	addKnown := func(value, configPath string) error {
		ref, ok, err := normalizeAssetReference(value, configPath)
		if err != nil {
			return err
		}
		if ok {
			refs = append(refs, ref)
		}
		return nil
	}

	if err := addKnown(bundle.Site.Header.Brand.Logo, bundle.SitePath+" -> header.brand.logo"); err != nil {
		return nil, err
	}
	for iconKey, iconPath := range bundle.Site.StoreIcons {
		if err := addKnown(iconPath, fmt.Sprintf("%s -> store_icons.%s", bundle.SitePath, iconKey)); err != nil {
			return nil, err
		}
	}
	for i, link := range bundle.Site.Social.Links {
		if err := addKnown(link.IconImage, fmt.Sprintf("%s -> social.links[%d].icon_image", bundle.SitePath, i)); err != nil {
			return nil, err
		}
	}

	for _, pageFile := range bundle.Pages {
		pagePath := pageFile.Path
		if err := addKnown(pageFile.Page.SEO.OGImage, pagePath+" -> seo.og_image"); err != nil {
			return nil, err
		}

		if len(pageFile.Page.Hero) > 0 {
			var hero any
			if err := json.Unmarshal(pageFile.Page.Hero, &hero); err != nil {
				return nil, fmt.Errorf("%s -> hero: invalid json: %w", pagePath, err)
			}
			if err := collectAssetRefsFromAny(hero, pagePath+" -> hero", "hero", &refs); err != nil {
				return nil, err
			}
		}

		if err := collectAssetRefsFromWidgets(pageFile.Page.Widgets, pagePath+" -> widgets", &refs); err != nil {
			return nil, err
		}
	}

	return refs, nil
}

func collectAssetRefsFromWidgets(widgets []WidgetNode, basePath string, refs *[]bundleAssetReference) error {
	for i, widget := range widgets {
		widgetPath := fmt.Sprintf("%s[%d]", basePath, i)
		for key, raw := range widget.Props {
			propPath := widgetPath + ".props." + key

			if key == "children" {
				var children []WidgetNode
				if err := json.Unmarshal(raw, &children); err != nil {
					return fmt.Errorf("%s: invalid children array: %w", propPath, err)
				}
				if err := collectAssetRefsFromWidgets(children, propPath, refs); err != nil {
					return err
				}
				continue
			}

			var value any
			if err := json.Unmarshal(raw, &value); err != nil {
				return fmt.Errorf("%s: invalid json: %w", propPath, err)
			}
			if err := collectAssetRefsFromAny(value, propPath, key, refs); err != nil {
				return err
			}
		}
	}
	return nil
}

func collectAssetRefsFromAny(value any, path, key string, refs *[]bundleAssetReference) error {
	switch v := value.(type) {
	case map[string]any:
		keys := make([]string, 0, len(v))
		for childKey := range v {
			keys = append(keys, childKey)
		}
		sort.Strings(keys)
		for _, childKey := range keys {
			childValue := v[childKey]
			childPath := path + "." + childKey
			if err := collectAssetRefsFromAny(childValue, childPath, childKey, refs); err != nil {
				return err
			}
		}
	case []any:
		for i, item := range v {
			itemPath := fmt.Sprintf("%s[%d]", path, i)
			if err := collectAssetRefsFromAny(item, itemPath, key, refs); err != nil {
				return err
			}
		}
	case string:
		if !looksLikeAssetField(key) {
			return nil
		}
		ref, ok, err := normalizeAssetReference(v, path)
		if err != nil {
			return err
		}
		if ok {
			*refs = append(*refs, ref)
		}
	}

	return nil
}

func looksLikeAssetField(key string) bool {
	k := strings.ToLower(strings.TrimSpace(key))
	if k == "" {
		return false
	}
	return strings.Contains(k, "image") ||
		strings.Contains(k, "icon") ||
		strings.Contains(k, "logo") ||
		strings.Contains(k, "photo") ||
		strings.Contains(k, "asset") ||
		k == "src" ||
		k == "cover"
}

func normalizeAssetReference(value, configPath string) (bundleAssetReference, bool, error) {
	raw := strings.TrimSpace(value)
	if raw == "" {
		return bundleAssetReference{}, false, nil
	}
	if isExternalOrSpecialHref(raw) {
		return bundleAssetReference{}, false, nil
	}

	normalized := filepath.ToSlash(strings.TrimPrefix(raw, "./"))
	if !strings.HasPrefix(normalized, "assets/") {
		return bundleAssetReference{}, false, fmt.Errorf(`%s: local asset path must start with "assets/" (got %q)`, configPath, raw)
	}
	if normalized == "assets/" || normalized == "assets" {
		return bundleAssetReference{}, false, fmt.Errorf(`%s: local asset path must reference a file under "assets/" (got %q)`, configPath, raw)
	}
	return bundleAssetReference{
		WebPath:    normalized,
		ConfigPath: configPath,
	}, true, nil
}

func copyReferencedSiteAssets(bundle SiteBundle, targetDir string) error {
	refs, err := collectBundleAssetReferences(bundle)
	if err != nil {
		return err
	}

	seen := map[string]string{}
	for _, ref := range refs {
		if _, ok := seen[ref.WebPath]; ok {
			continue
		}
		seen[ref.WebPath] = ref.ConfigPath
	}

	paths := make([]string, 0, len(seen))
	for webPath := range seen {
		paths = append(paths, webPath)
	}
	sort.Strings(paths)

	for _, webPath := range paths {
		configPath := seen[webPath]
		srcAbs, relOut, err := resolveAssetUnderSiteBundle(bundle.SiteDir, webPath)
		if err != nil {
			return fmt.Errorf("%s: %w", configPath, err)
		}
		if _, err := os.Stat(srcAbs); err != nil {
			if os.IsNotExist(err) {
				return fmt.Errorf("%s: referenced asset does not exist: %q", configPath, webPath)
			}
			return fmt.Errorf("%s: cannot access referenced asset %q: %w", configPath, webPath, err)
		}
		dst := filepath.Join(targetDir, filepath.FromSlash(relOut))
		if err := copyFile(srcAbs, dst); err != nil {
			return fmt.Errorf("%s: cannot copy asset %q: %w", configPath, webPath, err)
		}
	}

	return nil
}

func resolveAssetUnderSiteBundle(siteDir, webPath string) (srcAbs string, relOut string, err error) {
	p := strings.TrimSpace(webPath)
	if p == "" {
		return "", "", fmt.Errorf("empty asset path")
	}
	p = filepath.ToSlash(strings.TrimPrefix(p, "./"))
	if strings.Contains(p, "..") {
		return "", "", fmt.Errorf("invalid asset path %q", webPath)
	}
	if !strings.HasPrefix(p, "assets/") {
		return "", "", fmt.Errorf(`invalid asset path %q (must start with "assets/")`, webPath)
	}

	assetsRoot, err := filepath.Abs(filepath.Join(siteDir, "assets"))
	if err != nil {
		return "", "", err
	}
	localRel := strings.TrimPrefix(p, "assets/")
	cleanLocalRel := filepath.Clean(filepath.FromSlash(localRel))
	if cleanLocalRel == "." || strings.HasPrefix(cleanLocalRel, "..") {
		return "", "", fmt.Errorf("invalid asset path %q", webPath)
	}
	srcAbs, err = filepath.Abs(filepath.Join(assetsRoot, cleanLocalRel))
	if err != nil {
		return "", "", err
	}
	checkRel, err := filepath.Rel(assetsRoot, srcAbs)
	if err != nil || checkRel == "." || strings.HasPrefix(checkRel, "..") {
		return "", "", fmt.Errorf("asset path escapes site assets root: %q", webPath)
	}

	relOut = "assets/" + filepath.ToSlash(checkRel)
	return srcAbs, relOut, nil
}
