package main

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestBuildRenderedPageDataAppliesMergeModel(t *testing.T) {
	footerEnabled := true
	bundle := SiteBundle{
		SitePath: "content/demo/site.json",
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
				Path: "content/demo/pages/home.json",
				Page: PageConfig{
					Slug: "",
				},
			},
			{
				Path: "content/demo/pages/about.json",
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

	widgetTpl, err := loadWidgetTemplates("Template")
	if err != nil {
		t.Fatalf("loadWidgetTemplates: %v", err)
	}
	data, err := buildRenderedPageData(bundle, bundle.Pages[1], aboutRoute, routes, widgetTpl)
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
	bundle := SiteBundle{
		SitePath: "content/demo/site.json",
		Site: SiteConfig{
			SiteID: "demo-site",
			Footer: FooterConfig{},
		},
		Pages: []SitePageFile{
			{
				Path: "content/demo/pages/home.json",
				Page: PageConfig{
					Slug: "",
					Widgets: []WidgetNode{{
						Type: "intro",
						Props: map[string]json.RawMessage{
							"title": json.RawMessage(`"Home"`),
						},
					}},
				},
			},
			{Path: "content/demo/pages/about.json", Page: PageConfig{Slug: "about", Widgets: []WidgetNode{}}},
		},
	}

	outDir := t.TempDir()
	if err := renderSiteBundle(bundle, outDir, "Template"); err != nil {
		t.Fatalf("renderSiteBundle failed: %v", err)
	}

	if _, err := os.Stat(filepath.Join(outDir, "index.html")); err != nil {
		t.Fatalf("missing root index.html: %v", err)
	}
	if _, err := os.Stat(filepath.Join(outDir, "about", "index.html")); err != nil {
		t.Fatalf("missing about/index.html: %v", err)
	}
	data, err := os.ReadFile(filepath.Join(outDir, "index.html"))
	if err != nil {
		t.Fatalf("read generated index.html: %v", err)
	}
	if !strings.Contains(string(data), `class="intro section`) {
		t.Fatalf("expected rendered intro section in index.html, got: %s", string(data))
	}
}

func TestRenderKometaSiteBundleSmoke(t *testing.T) {
	bundle, _, err := loadSiteBundle(filepath.Join("content", "kometa"))
	if err != nil {
		t.Fatalf("load kometa bundle: %v", err)
	}

	outDir := t.TempDir()
	templateDir := filepath.Join("Template")
	if err := copyTemplateStaticAssets(templateDir, outDir); err != nil {
		t.Fatalf("copyTemplateStaticAssets: %v", err)
	}
	if err := copyReferencedSiteAssets(bundle, outDir); err != nil {
		t.Fatalf("copyReferencedSiteAssets: %v", err)
	}
	if err := renderSiteBundle(bundle, outDir, templateDir); err != nil {
		t.Fatalf("renderSiteBundle: %v", err)
	}

	htmlBytes, err := os.ReadFile(filepath.Join(outDir, "index.html"))
	if err != nil {
		t.Fatalf("read index.html: %v", err)
	}
	html := string(htmlBytes)
	if strings.Contains(html, "ZgotmplZ") {
		t.Fatal("generated HTML must not contain html/template CSS failure marker ZgotmplZ")
	}
	if !strings.Contains(html, `--font-heading:`) || !strings.Contains(html, "Quicksand") {
		snip := html
		if len(snip) > 800 {
			snip = snip[:800]
		}
		t.Fatalf("expected real font stack in <style> :root, got snippet: %s", snip)
	}
	for _, needle := range []string{
		`id="site-widgets-config"`,
		`split-widget--single`,
		`data-catalog-carousel`,
		`id="vacancies"`,
		`id="apps"`,
	} {
		if !strings.Contains(html, needle) {
			t.Fatalf("expected %q in rendered index.html", needle)
		}
	}
}
