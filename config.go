package main

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

func loadConfig(path string) (Config, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return Config{}, fmt.Errorf("cannot read config %q: %w", path, err)
	}

	var config Config
	if err := json.Unmarshal(data, &config); err != nil {
		return Config{}, fmt.Errorf("invalid config json %q: %w", path, err)
	}
	return config, nil
}

func loadSiteBundle(siteDir string) (SiteBundle, []ConfigWarning, error) {
	cleanSiteDir := filepath.Clean(siteDir)
	bundle := SiteBundle{
		SiteDir: cleanSiteDir,
	}

	sitePath := filepath.Join(cleanSiteDir, "site.json")
	bundle.SitePath = sitePath

	siteConfig, siteWarnings, err := loadSiteConfig(sitePath)
	if err != nil {
		return SiteBundle{}, nil, err
	}
	bundle.Site = siteConfig

	pagePaths, err := filepath.Glob(filepath.Join(cleanSiteDir, "pages", "*.json"))
	if err != nil {
		return SiteBundle{}, nil, fmt.Errorf("cannot list page configs in %q: %w", filepath.Join(cleanSiteDir, "pages"), err)
	}
	sort.Strings(pagePaths)

	warnings := append([]ConfigWarning{}, siteWarnings...)
	for _, pagePath := range pagePaths {
		pageFile, pageWarnings, err := loadPageConfig(pagePath)
		if err != nil {
			return SiteBundle{}, nil, err
		}
		warnings = append(warnings, pageWarnings...)
		bundle.Pages = append(bundle.Pages, pageFile)
	}

	if err := validateSiteBundle(bundle); err != nil {
		return SiteBundle{}, warnings, err
	}

	return bundle, warnings, nil
}

func loadSiteConfig(path string) (SiteConfig, []ConfigWarning, error) {
	var site SiteConfig
	rawKeys, err := decodeJSONObjectFile(path, &site)
	if err != nil {
		return SiteConfig{}, nil, err
	}

	warnings := unknownTopLevelKeyWarnings(path, rawKeys, siteTopLevelKeys)
	return site, warnings, nil
}

func loadPageConfig(path string) (SitePageFile, []ConfigWarning, error) {
	var page PageConfig
	rawKeys, err := decodeJSONObjectFile(path, &page)
	if err != nil {
		return SitePageFile{}, nil, err
	}

	pageFile := SitePageFile{
		Path:       path,
		Page:       page,
		HasSlug:    hasTopLevelKey(rawKeys, "slug"),
		HasWidgets: hasTopLevelKey(rawKeys, "widgets"),
	}

	warnings := unknownTopLevelKeyWarnings(path, rawKeys, pageTopLevelKeys)
	return pageFile, warnings, nil
}

var siteTopLevelKeys = keySet(
	"site_id",
	"output_folder",
	"theme",
	"typography",
	"game_store_icons",
	"game_subscribe",
	"social",
	"header",
	"footer",
	"base_url",
)

var pageTopLevelKeys = keySet(
	"slug",
	"widgets",
	"title",
	"seo",
	"hero",
	"layout",
)

func decodeJSONObjectFile(path string, target any) (map[string]json.RawMessage, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("cannot read config %q: %w", path, err)
	}

	var rawKeys map[string]json.RawMessage
	if err := json.Unmarshal(data, &rawKeys); err != nil {
		return nil, fmt.Errorf("invalid config json %q: %w", path, err)
	}

	if err := json.Unmarshal(data, target); err != nil {
		return nil, fmt.Errorf("invalid config json %q: %w", path, err)
	}

	return rawKeys, nil
}

func unknownTopLevelKeyWarnings(path string, rawKeys map[string]json.RawMessage, allowed map[string]struct{}) []ConfigWarning {
	if len(rawKeys) == 0 {
		return nil
	}

	unknown := make([]string, 0)
	for key := range rawKeys {
		if _, ok := allowed[key]; ok {
			continue
		}
		unknown = append(unknown, key)
	}
	sort.Strings(unknown)

	warnings := make([]ConfigWarning, 0, len(unknown))
	for _, key := range unknown {
		warnings = append(warnings, ConfigWarning{
			FilePath: path,
			Key:      key,
		})
	}
	return warnings
}

func keySet(keys ...string) map[string]struct{} {
	set := make(map[string]struct{}, len(keys))
	for _, key := range keys {
		set[key] = struct{}{}
	}
	return set
}

func hasTopLevelKey(rawKeys map[string]json.RawMessage, key string) bool {
	_, ok := rawKeys[key]
	return ok
}

func validateSiteBundle(bundle SiteBundle) error {
	if strings.TrimSpace(bundle.Site.SiteID) == "" {
		return fmt.Errorf(`%s -> "site_id" is required and must not be empty`, bundle.SitePath)
	}
	if _, err := validatedOutputFolderFor(bundle.Site.OutputFolder, bundle.SitePath); err != nil {
		return err
	}

	slugOwner := make(map[string]string)
	for _, pageFile := range bundle.Pages {
		if !pageFile.HasSlug {
			return fmt.Errorf(`%s -> "slug" is required`, pageFile.Path)
		}
		if !pageFile.HasWidgets {
			return fmt.Errorf(`%s -> "widgets" is required`, pageFile.Path)
		}
		if pageFile.Page.Widgets == nil {
			return fmt.Errorf(`%s -> "widgets" must be an array`, pageFile.Path)
		}

		slug := pageFile.Page.Slug
		if otherPath, exists := slugOwner[slug]; exists {
			return fmt.Errorf(`%s -> duplicate slug %q (already used in %s)`, pageFile.Path, slug, otherPath)
		}
		slugOwner[slug] = pageFile.Path
	}

	return nil
}

func validatedOutputFolder(name string) (string, error) {
	return validatedOutputFolderFor(name, "config.json")
}

func validatedOutputFolderFor(name, source string) (string, error) {
	s := strings.TrimSpace(name)
	if s == "" {
		return "", fmt.Errorf(`%s: "output_folder" is required and must not be empty`, source)
	}
	if strings.ContainsAny(s, `/\`) {
		return "", fmt.Errorf(`%s: "output_folder" must not contain path separators (got %q)`, source, s)
	}
	if s == "." || s == ".." || strings.Contains(s, "..") {
		return "", fmt.Errorf(`%s: invalid "output_folder" %q`, source, s)
	}
	return s, nil
}
