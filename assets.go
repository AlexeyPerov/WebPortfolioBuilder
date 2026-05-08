package main

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"image"
	_ "image/gif"
	_ "image/jpeg"
	_ "image/png"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

var pngFileSignature = []byte{0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A}

func readPNGPixelHeight(path string) (int, error) {
	b, err := os.ReadFile(path)
	if err != nil {
		return 0, err
	}
	if len(b) < 24 {
		return 0, errors.New("file too small for png")
	}
	if !bytes.Equal(b[:8], pngFileSignature) {
		return 0, errors.New("not a png signature")
	}
	if string(b[12:16]) != "IHDR" {
		return 0, errors.New("png missing IHDR")
	}
	h := int(b[20])<<24 | int(b[21])<<16 | int(b[22])<<8 | int(b[23])
	if h <= 0 || h > 100_000 {
		return 0, errors.New("invalid png height")
	}
	return h, nil
}

const defaultGameStoreIconHeightPx = 52

// gameStoreIconDisplayScale is applied to the measured PNG height (30% smaller → 70% size).
const gameStoreIconDisplayScale = 0.7

func scaledGameStoreIconHeightPx(measured int) int {
	if measured <= 0 {
		return 0
	}
	return int(float64(measured)*gameStoreIconDisplayScale + 0.5)
}

func gameStoreIconHeightCSS(projectRoot string, icons GameStoreIcons) string {
	def := scaledGameStoreIconHeightPx(defaultGameStoreIconHeightPx)
	gp := strings.TrimSpace(icons["google_play"])
	if gp == "" {
		return fmt.Sprintf("%dpx", def)
	}
	abs, _, err := resolveAssetUnderProject(projectRoot, gp)
	if err != nil {
		return fmt.Sprintf("%dpx", def)
	}
	h, err := readPNGPixelHeight(abs)
	if err != nil {
		return fmt.Sprintf("%dpx", def)
	}
	return fmt.Sprintf("%dpx", scaledGameStoreIconHeightPx(h))
}

const gameIconDisplayScale = 0.8

func gameIconImgSizeAttrs(projectRoot, webPath string) string {
	webPath = strings.TrimSpace(webPath)
	if webPath == "" {
		return ""
	}
	abs, _, err := resolveAssetUnderProject(projectRoot, webPath)
	if err != nil {
		return ""
	}
	f, err := os.Open(abs)
	if err != nil {
		return ""
	}
	defer f.Close()
	cfg, _, err := image.DecodeConfig(f)
	if err != nil {
		return ""
	}
	w := int(float64(cfg.Width)*gameIconDisplayScale + 0.5)
	h := int(float64(cfg.Height)*gameIconDisplayScale + 0.5)
	if w < 1 {
		w = 1
	}
	if h < 1 {
		h = 1
	}
	return fmt.Sprintf(` width="%d" height="%d"`, w, h)
}

func copyConfigAssets(projectRoot, siteRoot string, config Config) error {
	paths := collectAssetPaths(config)
	for _, webPath := range paths {
		if err := copyProjectAsset(projectRoot, siteRoot, webPath); err != nil {
			return err
		}
	}
	return nil
}

func collectAssetPaths(config Config) []string {
	seen := map[string]struct{}{}
	var out []string

	add := func(p string) {
		p = strings.TrimSpace(p)
		if p == "" {
			return
		}
		p = strings.TrimPrefix(p, "./")
		key := filepath.ToSlash(p)
		if _, ok := seen[key]; ok {
			return
		}
		seen[key] = struct{}{}
		out = append(out, p)
	}

	if config.Content != nil {
		add(config.Content["site_icon"])
		add(config.Content["header_logo_image"])
		add(config.Content["cover_image"])
	}
	gsi := config.GameStoreIcons.withDefaults()
	for _, path := range gsi {
		add(path)
	}
	for _, link := range config.Social.resolvedSocialLinks() {
		add(link.IconImage)
	}
	for _, p := range config.Photos {
		add(p)
	}
	for _, o := range config.Offers {
		add(o.Image)
	}
	for _, g := range config.Games {
		add(g.Image)
		add(g.HeaderImage)
		for _, p := range g.SwiperImages {
			add(p)
		}
		for _, sl := range g.StoreLinks {
			add(sl.IconImage)
		}
	}
	return out
}

func copyProjectAsset(projectRoot, siteRoot, webPath string) error {
	srcAbs, rel, err := resolveAssetUnderProject(projectRoot, webPath)
	if err != nil {
		return err
	}
	if _, err := os.Stat(srcAbs); err != nil {
		if os.IsNotExist(err) {
			return fmt.Errorf("asset not found (referenced in config): %s", webPath)
		}
		return fmt.Errorf("asset %s: %w", webPath, err)
	}
	dst := filepath.Join(siteRoot, rel)
	return copyFile(srcAbs, dst)
}

func resolveAssetUnderProject(projectRoot, webPath string) (srcAbs string, rel string, err error) {
	webPath = strings.TrimSpace(webPath)
	if webPath == "" {
		return "", "", errors.New("empty asset path")
	}
	webPath = strings.TrimPrefix(webPath, "./")
	if strings.Contains(filepath.ToSlash(webPath), "..") {
		return "", "", fmt.Errorf("invalid asset path: %q", webPath)
	}

	rel = filepath.FromSlash(webPath)

	absRoot, err := filepath.Abs(filepath.Clean(projectRoot))
	if err != nil {
		return "", "", err
	}
	joined := filepath.Join(absRoot, rel)
	absSrc, err := filepath.Abs(filepath.Clean(joined))
	if err != nil {
		return "", "", err
	}

	relOut, err := filepath.Rel(absRoot, absSrc)
	if err != nil || strings.HasPrefix(relOut, "..") {
		return "", "", fmt.Errorf("asset path escapes project root: %q", webPath)
	}

	return absSrc, relOut, nil
}

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
	for iconKey, iconPath := range bundle.Site.GameStoreIcons {
		if err := addKnown(iconPath, fmt.Sprintf("%s -> game_store_icons.%s", bundle.SitePath, iconKey)); err != nil {
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
		return "", "", errors.New("empty asset path")
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
