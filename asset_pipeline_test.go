package main

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestCollectBundleAssetReferencesRejectsNonAssetsPrefix(t *testing.T) {
	siteDir := t.TempDir()
	bundle := SiteBundle{
		SiteDir:  siteDir,
		SitePath: filepath.Join(siteDir, "site.json"),
		Site: SiteConfig{
			Header: HeaderConfig{
				Brand: HeaderBrand{Logo: "Images/logo.png"},
			},
		},
	}

	_, err := collectBundleAssetReferences(bundle)
	if err == nil {
		t.Fatal("expected non-assets prefix validation error")
	}
	if !strings.Contains(err.Error(), `header.brand.logo`) {
		t.Fatalf("expected config path in error, got: %v", err)
	}
}

func TestCopyReferencedSiteAssetsRejectsEscapePath(t *testing.T) {
	siteDir := t.TempDir()
	pagePath := filepath.Join(siteDir, "pages", "home.json")
	if err := os.MkdirAll(filepath.Dir(pagePath), 0o755); err != nil {
		t.Fatalf("mkdir pages: %v", err)
	}

	widget := WidgetNode{
		Type: "images_grid",
		Props: map[string]json.RawMessage{
			"image": mustRawJSON(t, "assets/../secrets.png"),
		},
	}
	bundle := SiteBundle{
		SiteDir:  siteDir,
		SitePath: filepath.Join(siteDir, "site.json"),
		Pages: []SitePageFile{
			{
				Path: pagePath,
				Page: PageConfig{
					Slug:    "",
					Widgets: []WidgetNode{widget},
				},
			},
		},
	}

	err := copyReferencedSiteAssets(bundle, t.TempDir())
	if err == nil {
		t.Fatal("expected escape validation error")
	}
	if !strings.Contains(err.Error(), "widgets[0].props.image") {
		t.Fatalf("expected widget config path in error, got: %v", err)
	}
}

func TestCopyReferencedSiteAssetsCopiesDedupedRecursiveReferences(t *testing.T) {
	siteDir := t.TempDir()
	assetsDir := filepath.Join(siteDir, "assets")
	if err := os.MkdirAll(filepath.Join(assetsDir, "icons"), 0o755); err != nil {
		t.Fatalf("mkdir assets: %v", err)
	}
	writeTestAsset(t, filepath.Join(assetsDir, "cover.png"))
	writeTestAsset(t, filepath.Join(assetsDir, "icons", "badge.png"))

	childWidget := WidgetNode{
		Type: "column",
		Props: map[string]json.RawMessage{
			"icon_image": mustRawJSON(t, "assets/icons/badge.png"),
		},
	}
	rootWidget := WidgetNode{
		Type: "row",
		Props: map[string]json.RawMessage{
			"image":    mustRawJSON(t, "assets/cover.png"),
			"cards":    mustRawJSON(t, []map[string]string{{"image": "assets/cover.png"}}),
			"children": mustRawJSON(t, []WidgetNode{childWidget}),
		},
	}

	pagePath := filepath.Join(siteDir, "pages", "home.json")
	bundle := SiteBundle{
		SiteDir:  siteDir,
		SitePath: filepath.Join(siteDir, "site.json"),
		Pages: []SitePageFile{
			{
				Path: pagePath,
				Page: PageConfig{
					Slug: "",
					SEO: PageSEO{
						OGImage: "assets/cover.png",
					},
					Widgets: []WidgetNode{rootWidget},
				},
			},
		},
	}

	outDir := t.TempDir()
	if err := copyReferencedSiteAssets(bundle, outDir); err != nil {
		t.Fatalf("copyReferencedSiteAssets failed: %v", err)
	}

	if _, err := os.Stat(filepath.Join(outDir, "assets", "cover.png")); err != nil {
		t.Fatalf("expected copied cover image: %v", err)
	}
	if _, err := os.Stat(filepath.Join(outDir, "assets", "icons", "badge.png")); err != nil {
		t.Fatalf("expected copied nested icon image: %v", err)
	}
}

func mustRawJSON(t *testing.T, value any) json.RawMessage {
	t.Helper()
	b, err := json.Marshal(value)
	if err != nil {
		t.Fatalf("marshal raw json: %v", err)
	}
	return json.RawMessage(b)
}

func writeTestAsset(t *testing.T, path string) {
	t.Helper()
	if err := os.WriteFile(path, []byte("asset"), 0o644); err != nil {
		t.Fatalf("write test asset %s: %v", path, err)
	}
}
