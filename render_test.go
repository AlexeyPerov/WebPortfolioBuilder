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
	data, _, err := buildRenderedPageData(bundle, bundle.Pages[1], aboutRoute, routes, widgetTpl)
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

func TestBuildRenderedPageDataEmitsSocialMetaFields(t *testing.T) {
	bundle := SiteBundle{
		SitePath: "content/demo/site.json",
		Site: SiteConfig{
			SiteID:  "demo-site",
			BaseURL: "https://example.com/repo",
			Theme: map[string]string{
				"accent": "#3296ed",
			},
		},
		Pages: []SitePageFile{
			{
				Path: "content/demo/pages/home.json",
				Page: PageConfig{
					Slug:  "",
					Title: "Demo Home",
					SEO: PageSEO{
						Description: "A polished demo portfolio site.",
						OGImage:     "assets/cover.png",
					},
				},
			},
		},
	}

	routes, err := buildRouteIndex(bundle)
	if err != nil {
		t.Fatalf("buildRouteIndex error: %v", err)
	}
	widgetTpl, err := loadWidgetTemplates("Template")
	if err != nil {
		t.Fatalf("loadWidgetTemplates: %v", err)
	}
	data, _, err := buildRenderedPageData(bundle, bundle.Pages[0], routes.Ordered[0], routes, widgetTpl)
	if err != nil {
		t.Fatalf("buildRenderedPageData error: %v", err)
	}
	if data.MetaDescription != "A polished demo portfolio site." {
		t.Fatalf("unexpected meta description: %q", data.MetaDescription)
	}
	if data.CanonicalURL != "https://example.com/repo/" {
		t.Fatalf("unexpected canonical url: %q", data.CanonicalURL)
	}
	if data.OpenGraphImage != "https://example.com/repo/assets/cover.png" {
		t.Fatalf("unexpected og image: %q", data.OpenGraphImage)
	}
	if data.ThemeColor != "#3296ed" {
		t.Fatalf("unexpected theme color: %q", data.ThemeColor)
	}
	if data.TwitterCard != "summary_large_image" {
		t.Fatalf("unexpected twitter card: %q", data.TwitterCard)
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
	if _, err := renderSiteBundle(bundle, outDir, "Template"); err != nil {
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
	if strings.Contains(string(data), `catalog-carousel.js`) {
		t.Fatal("intro-only page must not load catalog-carousel.js")
	}
	if strings.Contains(string(data), `split-widget.js`) {
		t.Fatal("intro-only page must not load split-widget.js")
	}
	if strings.Contains(string(data), `image-lightbox.js`) {
		t.Fatal("intro-only page must not load image-lightbox.js")
	}
	if !strings.Contains(string(data), `scroll-reveal.js`) {
		t.Fatal("intro-only page must load scroll-reveal.js")
	}
}

func TestRenderDemoSiteBundleSmoke(t *testing.T) {
	bundle, _, err := loadSiteBundle(filepath.Join("content", "demo"))
	if err != nil {
		t.Fatalf("load demo bundle: %v", err)
	}

	outDir := t.TempDir()
	templateDir := filepath.Join("Template")
	if err := copyTemplateStaticAssets(templateDir, outDir); err != nil {
		t.Fatalf("copyTemplateStaticAssets: %v", err)
	}
	if err := copyReferencedSiteAssets(bundle, outDir); err != nil {
		t.Fatalf("copyReferencedSiteAssets: %v", err)
	}
	if _, err := renderSiteBundle(bundle, outDir, templateDir); err != nil {
		t.Fatalf("renderSiteBundle: %v", err)
	}

	if _, err := os.Stat(filepath.Join(outDir, "index.html")); err != nil {
		t.Fatalf("missing index.html: %v", err)
	}
	if _, err := os.Stat(filepath.Join(outDir, "about", "index.html")); err != nil {
		t.Fatalf("missing about/index.html: %v", err)
	}
	for _, page := range []string{"layouts", "gallery", "apps", "careers"} {
		if _, err := os.Stat(filepath.Join(outDir, page, "index.html")); err != nil {
			t.Fatalf("missing %s/index.html: %v", page, err)
		}
	}

	htmlBytes, err := os.ReadFile(filepath.Join(outDir, "index.html"))
	if err != nil {
		t.Fatalf("read index.html: %v", err)
	}
	html := string(htmlBytes)
	for _, needle := range []string{
		`data-widget-type="project_grid"`,
		`href="about/"`,
		`href="layouts/"`,
		`project-card__cta`,
		`id="offers"`,
		`class="photos section section-gradient scroll-reveal"`,
	} {
		if !strings.Contains(html, needle) {
			t.Fatalf("expected %q in demo index.html", needle)
		}
	}

	appsHTML, err := os.ReadFile(filepath.Join(outDir, "apps", "index.html"))
	if err != nil {
		t.Fatalf("read apps/index.html: %v", err)
	}
	apps := string(appsHTML)
	for _, needle := range []string{
		`data-widget-type="apps_showcase"`,
		`data-widget-type="project_grid"`,
	} {
		if !strings.Contains(apps, needle) {
			t.Fatalf("expected %q in demo apps/index.html", needle)
		}
	}

	galleryHTML, err := os.ReadFile(filepath.Join(outDir, "gallery", "index.html"))
	if err != nil {
		t.Fatalf("read gallery/index.html: %v", err)
	}
	gallery := string(galleryHTML)
	for _, needle := range []string{
		`data-widget-type="media_swiper"`,
		`class="photos section section-gradient scroll-reveal"`,
	} {
		if !strings.Contains(gallery, needle) {
			t.Fatalf("expected %q in demo gallery/index.html", needle)
		}
	}

	careersHTML, err := os.ReadFile(filepath.Join(outDir, "careers", "index.html"))
	if err != nil {
		t.Fatalf("read careers/index.html: %v", err)
	}
	if !strings.Contains(string(careersHTML), `id="vacancies"`) {
		t.Fatal(`expected id="vacancies" in demo careers/index.html`)
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
	if _, err := renderSiteBundle(bundle, outDir, templateDir); err != nil {
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
		`scroll-reveal.js`,
		`catalog-carousel.js`,
		`split-widget.js`,
		`split-widget--single`,
		`data-catalog-carousel`,
		`id="vacancies"`,
		`id="apps"`,
		`<meta name="description" content="Kometa.Games is a mobile game studio`,
		`<meta property="og:title" content="Kometa.Games">`,
		`<meta property="og:url" content="https://YOUR-GITHUB-USER.github.io/YOUR-REPO-NAME/">`,
		`<meta name="theme-color" content="#3296ed">`,
		`data-image-lightbox`,
		`image-lightbox.js`,
		`alt="Alec Monopoly pop art print in the Kometa office"`,
	} {
		if !strings.Contains(html, needle) {
			t.Fatalf("expected %q in rendered index.html", needle)
		}
	}
}
