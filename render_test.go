package main

import (
	"os"
	"path/filepath"
	"testing"
)

func TestBuildRenderedPageDataAppliesMergeModel(t *testing.T) {
	footerEnabled := true
	bundle := SiteBundle{
		SitePath: "sites/demo/site.json",
		Site: SiteConfig{
			SiteID:  "demo-site",
			BaseURL: "https://example.com/repo",
			Header: HeaderConfig{
				Nav: []NavItem{
					{Label: "Home", Href: ""},
					{Label: "About", Href: "about"},
				},
			},
			Footer: FooterConfig{
				Enabled:      &footerEnabled,
				SectionTitle: "Contact",
			},
		},
		Pages: []SitePageFile{
			{
				Path: "sites/demo/pages/home.json",
				Page: PageConfig{
					Slug: "",
				},
			},
			{
				Path: "sites/demo/pages/about.json",
				Page: PageConfig{
					Slug:   "about",
					Layout: PageLayout{HideHeader: true, HideFooter: true},
				},
			},
		},
	}

	routes, err := buildRouteIndex(bundle)
	if err != nil {
		t.Fatalf("buildRouteIndex error: %v", err)
	}

	var aboutRoute PageRoute
	for _, r := range routes.Ordered {
		if r.Slug == "about" {
			aboutRoute = r
			break
		}
	}

	data, err := buildRenderedPageData(bundle, bundle.Pages[1], aboutRoute, routes)
	if err != nil {
		t.Fatalf("buildRenderedPageData error: %v", err)
	}

	if data.Title != "demo-site" {
		t.Fatalf("expected fallback title demo-site, got %q", data.Title)
	}
	if data.ShowHeader {
		t.Fatal("expected header hidden by page layout")
	}
	if data.ShowFooter {
		t.Fatal("expected footer hidden by page layout")
	}
	if data.CanonicalURL != "https://example.com/repo/about/" {
		t.Fatalf("unexpected canonical url: %q", data.CanonicalURL)
	}
}

func TestRenderSiteBundleWritesOneHtmlPerRoute(t *testing.T) {
	templateDir := t.TempDir()
	layoutPath := filepath.Join(templateDir, "layout.html")
	layout := `<!doctype html><html><head><title>{{.Title}}</title></head><body>{{.MainContentHTML}}</body></html>`
	if err := os.WriteFile(layoutPath, []byte(layout), 0o644); err != nil {
		t.Fatalf("write layout: %v", err)
	}

	bundle := SiteBundle{
		SitePath: "sites/demo/site.json",
		Site: SiteConfig{
			SiteID: "demo-site",
			Footer: FooterConfig{},
		},
		Pages: []SitePageFile{
			{Path: "sites/demo/pages/home.json", Page: PageConfig{Slug: "", Widgets: []WidgetNode{}}},
			{Path: "sites/demo/pages/about.json", Page: PageConfig{Slug: "about", Widgets: []WidgetNode{}}},
		},
	}

	outDir := t.TempDir()
	if err := renderSiteBundle(bundle, outDir, templateDir); err != nil {
		t.Fatalf("renderSiteBundle failed: %v", err)
	}

	if _, err := os.Stat(filepath.Join(outDir, "index.html")); err != nil {
		t.Fatalf("missing root index.html: %v", err)
	}
	if _, err := os.Stat(filepath.Join(outDir, "about", "index.html")); err != nil {
		t.Fatalf("missing about/index.html: %v", err)
	}
}
