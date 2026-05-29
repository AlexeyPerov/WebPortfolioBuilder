package main

import (
	"encoding/json"
	"html/template"
	"strings"
	"testing"
)

func testWidgetTpl(t *testing.T) *template.Template {
	t.Helper()
	tpl, err := loadWidgetTemplates("Template")
	if err != nil {
		t.Fatalf("loadWidgetTemplates: %v", err)
	}
	return tpl
}

func testRenderCtx(t *testing.T, pagePath string) *widgetRenderContext {
	t.Helper()
	return &widgetRenderContext{
		PagePath:  pagePath,
		Site:      SiteConfig{},
		Route:     PageRoute{Slug: "", DirRelPath: ""},
		Routes:    RouteIndex{BySlug: map[string]PageRoute{"": {Slug: "", DirRelPath: ""}}},
		WidgetTpl: testWidgetTpl(t),
	}
}

func TestRenderWidgetTreeUnknownTypeFailsWithPath(t *testing.T) {
	ctx := testRenderCtx(t, "content/demo/pages/home.json")
	widgets := []WidgetNode{
		{Type: "unknown"},
	}
	_, _, err := renderWidgetTree(ctx, widgets)
	if err == nil {
		t.Fatal("expected unknown type error")
	}
	if !strings.Contains(err.Error(), "widgets[0].type") {
		t.Fatalf("expected widget path in error, got: %v", err)
	}
}

func TestRenderWidgetTreeRejectsColumnsAlias(t *testing.T) {
	ctx := testRenderCtx(t, "content/demo/pages/home.json")
	widgets := []WidgetNode{
		{Type: "columns"},
	}
	_, _, err := renderWidgetTree(ctx, widgets)
	if err == nil {
		t.Fatal("expected columns alias rejection")
	}
	if !strings.Contains(err.Error(), `unknown widget type "columns"`) {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestRenderWidgetTreeLeafChildrenFails(t *testing.T) {
	ctx := testRenderCtx(t, "content/demo/pages/home.json")
	widgets := []WidgetNode{
		{
			Type: "intro",
			Props: map[string]json.RawMessage{
				"children": mustWidgetRawJSON(t, []WidgetNode{{Type: "intro"}}),
			},
		},
	}
	_, _, err := renderWidgetTree(ctx, widgets)
	if err == nil {
		t.Fatal("expected leaf children error")
	}
	if !strings.Contains(err.Error(), "only layout widgets") {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestRenderWidgetTreeLayoutNeedsChildren(t *testing.T) {
	ctx := testRenderCtx(t, "content/demo/pages/home.json")
	widgets := []WidgetNode{
		{Type: "row"},
	}
	_, _, err := renderWidgetTree(ctx, widgets)
	if err == nil {
		t.Fatal("expected missing children error")
	}
	if !strings.Contains(err.Error(), ".props.children") {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestRenderWidgetTreeLayoutRecurses(t *testing.T) {
	ctx := testRenderCtx(t, "content/demo/pages/home.json")
	widgets := []WidgetNode{
		{
			Type: "row",
			Props: map[string]json.RawMessage{
				"children": mustWidgetRawJSON(t, []WidgetNode{
					{
						Type: "column",
						Props: map[string]json.RawMessage{
							"children": mustWidgetRawJSON(t, []WidgetNode{
								{
									Type: "intro",
									ID:   "intro1",
									Props: map[string]json.RawMessage{
										"title": mustWidgetRawJSON(t, "About us"),
									},
								},
							}),
						},
					},
				}),
			},
		},
	}
	out, _, err := renderWidgetTree(ctx, widgets)
	if err != nil {
		t.Fatalf("renderWidgetTree failed: %v", err)
	}
	html := string(out)
	if !strings.Contains(html, `class="widget-row section"`) {
		t.Fatalf("expected widget-row wrapper, got: %s", html)
	}
	if !strings.Contains(html, `class="container widget-row-inner"`) {
		t.Fatalf("expected widget-row-inner container, got: %s", html)
	}
	if !strings.Contains(html, `class="widget-column"`) {
		t.Fatalf("expected widget-column wrapper, got: %s", html)
	}
	if !strings.Contains(html, `class="intro section`) {
		t.Fatalf("expected intro section, got: %s", html)
	}
	if !strings.Contains(html, `id="intro_title"`) {
		t.Fatalf("expected intro heading id parity, got: %s", html)
	}
}

func TestGridRendersCustomMinColumnWidth(t *testing.T) {
	ctx := testRenderCtx(t, "content/demo/pages/home.json")
	widgets := []WidgetNode{
		{
			Type: "grid",
			Props: map[string]json.RawMessage{
				"min_column_width": mustWidgetRawJSON(t, "312px"),
				"children": mustWidgetRawJSON(t, []WidgetNode{
					{
						Type: "intro",
						Props: map[string]json.RawMessage{
							"title": mustWidgetRawJSON(t, "Hi"),
						},
					},
				}),
			},
		},
	}
	out, _, err := renderWidgetTree(ctx, widgets)
	if err != nil {
		t.Fatalf("renderWidgetTree failed: %v", err)
	}
	html := string(out)
	if !strings.Contains(html, "312px") {
		t.Fatalf("expected min column width in output, got: %s", html)
	}
}

func TestCoverBannerRequiresSrc(t *testing.T) {
	ctx := testRenderCtx(t, "content/demo/pages/home.json")
	widgets := []WidgetNode{{Type: "cover_banner", Props: map[string]json.RawMessage{}}}
	_, _, err := renderWidgetTree(ctx, widgets)
	if err == nil || !strings.Contains(err.Error(), "src") {
		t.Fatalf("expected src required error, got: %v", err)
	}
}

func TestCareersTabsSingleVacancyNoTabs(t *testing.T) {
	ctx := testRenderCtx(t, "content/demo/pages/home.json")
	widgets := []WidgetNode{
		{
			Type: "careers_tabs",
			Props: map[string]json.RawMessage{
				"title": mustWidgetRawJSON(t, "Careers"),
				"vacancies": mustWidgetRawJSON(t, []Vacancy{
					{
						Role:             "Designer",
						Requirements:     []string{"Portfolio"},
						Responsibilities: []string{"UI work"},
						Advantages:       []string{"Remote"},
						ApplyURL:         "https://example.com/apply",
					},
				}),
			},
		},
	}
	out, _, err := renderWidgetTree(ctx, widgets)
	if err != nil {
		t.Fatalf("renderWidgetTree failed: %v", err)
	}
	html := string(out)
	if strings.Contains(html, `data-split-widget`) {
		t.Fatalf("single vacancy should not emit data-split-widget: %s", html)
	}
	if strings.Contains(html, `split-widget__tab`) {
		t.Fatalf("single vacancy should not render tab buttons: %s", html)
	}
	if !strings.Contains(html, `split-widget--single`) || !strings.Contains(html, `vacancy-detail`) {
		t.Fatalf("expected single-vacancy panel markup: %s", html)
	}
}

func TestCareersTabsMultiVacancyHasTabs(t *testing.T) {
	ctx := testRenderCtx(t, "content/demo/pages/home.json")
	widgets := []WidgetNode{
		{
			Type: "careers_tabs",
			Props: map[string]json.RawMessage{
				"title": mustWidgetRawJSON(t, "Careers"),
				"vacancies": mustWidgetRawJSON(t, []Vacancy{
					{
						Role:             "Designer",
						Requirements:     []string{"Portfolio"},
						Responsibilities: []string{"UI work"},
						Advantages:       []string{"Remote"},
						ApplyURL:         "https://example.com/apply",
					},
					{
						Role:             "Engineer",
						Requirements:     []string{"Go experience"},
						Responsibilities: []string{"Backend work"},
						Advantages:       []string{"Flexible hours"},
						ApplyURL:         "https://example.com/apply2",
					},
				}),
			},
		},
	}
	out, _, err := renderWidgetTree(ctx, widgets)
	if err != nil {
		t.Fatalf("renderWidgetTree failed: %v", err)
	}
	html := string(out)
	if !strings.Contains(html, `data-split-widget`) {
		t.Fatalf("expected split widget marker: %s", html)
	}
	if !strings.Contains(html, `data-target="vacancy-0"`) || !strings.Contains(html, `id="vacancy-0"`) {
		t.Fatalf("expected vacancy tab wiring: %s", html)
	}
	if !strings.Contains(html, `data-target="vacancy-1"`) {
		t.Fatalf("expected second vacancy tab: %s", html)
	}
}

func TestAppsShowcaseRendersCardsAndSwiperAndStores(t *testing.T) {
	ctx := testRenderCtx(t, "content/demo/pages/home.json")
	ctx.Site = SiteConfig{
		StoreIcons: StoreIcons{
			"google_play": "assets/icons/googleplay.png",
		},
		SubscribeBlock: SubscribeBlock{
			Title: "Stay updated",
			Links: []SubscribeLink{
				{Label: "Telegram", URL: "https://t.me/example"},
			},
		},
	}
	widgets := []WidgetNode{
		{
			Type: "apps_showcase",
			Props: map[string]json.RawMessage{
				"section_title": mustWidgetRawJSON(t, "Our apps"),
				"apps": mustWidgetRawJSON(t, []CatalogApp{
					{
						Image:         "assets/icons/app.png",
						HeaderImage:   "assets/headers/app-header.png",
						Title:         "Kometa",
						Text1:         "First paragraph",
						Text2:         "Second paragraph",
						SwiperImages:  []string{"assets/swiper/1.png"},
						GooglePlayURL: "https://play.google.com/store/apps/details?id=kometa",
					},
					{
						Image: "assets/icons/app2.png",
						Title: "Kometa 2",
						StoreLinks: []StoreLink{
							{
								URL:       "https://example.com/store",
								AriaLabel: "Open store",
								Icon:      "google_play",
							},
						},
					},
				}),
			},
		},
	}
	out, _, err := renderWidgetTree(ctx, widgets)
	if err != nil {
		t.Fatalf("renderWidgetTree failed: %v", err)
	}
	html := string(out)
	if !strings.Contains(html, `data-widget-type="apps_showcase"`) {
		t.Fatalf("expected apps_showcase section marker, got: %s", html)
	}
	if !strings.Contains(html, `class="catalog-carousel" data-catalog-carousel`) {
		t.Fatalf("expected catalog carousel contract markup, got: %s", html)
	}
	if !strings.Contains(html, `class="catalog-store-btn catalog-store-btn--googleplay"`) {
		t.Fatalf("expected store badge button, got: %s", html)
	}
	if !strings.Contains(html, `class="catalog-app-card__subscribe"`) {
		t.Fatalf("expected subscribe block, got: %s", html)
	}
}

func TestAppsShowcaseHidesSubscribeWhenNoLinks(t *testing.T) {
	ctx := testRenderCtx(t, "content/demo/pages/home.json")
	ctx.Site = SiteConfig{
		StoreIcons: StoreIcons{"google_play": "assets/icons/googleplay.png"},
		SubscribeBlock: SubscribeBlock{
			Title: "Subscribe for news",
			Links: nil,
		},
	}
	widgets := []WidgetNode{
		{
			Type: "apps_showcase",
			Props: map[string]json.RawMessage{
				"apps": mustWidgetRawJSON(t, []CatalogApp{
					{
						Image:         "assets/icons/app.png",
						SwiperImages:  []string{"assets/swiper/1.png"},
						GooglePlayURL: "https://play.google.com/store/apps/details?id=example",
					},
				}),
			},
		},
	}
	out, _, err := renderWidgetTree(ctx, widgets)
	if err != nil {
		t.Fatalf("renderWidgetTree failed: %v", err)
	}
	if strings.Contains(string(out), `class="catalog-app-card__subscribe"`) {
		t.Fatalf("expected no subscribe block when links empty, got: %s", out)
	}
}

func TestAppsShowcaseEmptyStoreURLWarnsAndSkips(t *testing.T) {
	ctx := testRenderCtx(t, "content/kometa/pages/home.json")
	ctx.Site = SiteConfig{
		StoreIcons: StoreIcons{
			"google_play": "assets/gp-store-icon.png",
			"app_store":   "assets/appstore-store-icon.png",
			"amazon":      "assets/amazon-store-icon.png",
			"galaxy":      "assets/galaxy-store-icon.png",
		},
	}
	widgets := []WidgetNode{
		{
			Type: "apps_showcase",
			Props: map[string]json.RawMessage{
				"apps": json.RawMessage(`[{
		"image": "assets/icon.png",
		"swiper_images": ["assets/slide.png"],
		"google_play_url": "https://play.google.com/store/apps/details?id=one",
		"amazon_store_url": "",
		"galaxy_store_url": ""
	}]`),
			},
		},
	}
	out, warnings, err := renderWidgetTree(ctx, widgets)
	if err != nil {
		t.Fatalf("renderWidgetTree failed: %v", err)
	}
	html := string(out)
	if strings.Contains(html, `catalog-store-btn--amazon`) || strings.Contains(html, `catalog-store-btn--galaxy`) {
		t.Fatalf("expected no amazon/galaxy badges for empty URLs, got: %s", html)
	}
	if len(warnings) != 2 {
		t.Fatalf("expected 2 store URL warnings, got %d: %v", len(warnings), warnings)
	}
	for _, w := range warnings {
		if !strings.Contains(w.String(), "referenced but URL is empty") {
			t.Fatalf("unexpected warning: %s", w.String())
		}
	}
}

func TestAppsShowcaseRequiresApps(t *testing.T) {
	ctx := testRenderCtx(t, "content/demo/pages/home.json")
	widgets := []WidgetNode{
		{
			Type:  "apps_showcase",
			Props: map[string]json.RawMessage{},
		},
	}
	_, _, err := renderWidgetTree(ctx, widgets)
	if err == nil {
		t.Fatal("expected apps_showcase apps validation error")
	}
	if !strings.Contains(err.Error(), "widgets[0].props.apps") {
		t.Fatalf("expected path-aware apps error, got: %v", err)
	}
}

func TestAppsShowcaseAppImageRequiredPath(t *testing.T) {
	ctx := testRenderCtx(t, "content/demo/pages/home.json")
	widgets := []WidgetNode{
		{
			Type: "apps_showcase",
			Props: map[string]json.RawMessage{
				"apps": mustWidgetRawJSON(t, []CatalogApp{
					{Title: "No image"},
				}),
			},
		},
	}
	_, _, err := renderWidgetTree(ctx, widgets)
	if err == nil {
		t.Fatal("expected apps_showcase image validation error")
	}
	if !strings.Contains(err.Error(), "widgets[0].props.apps[0].image") {
		t.Fatalf("expected path-aware image error, got: %v", err)
	}
}

func TestRenderWidgetTreeSkipsDisabledWidgets(t *testing.T) {
	ctx := testRenderCtx(t, "content/demo/pages/home.json")
	disabled := false
	widgets := []WidgetNode{
		{
			Type: "intro",
			Props: map[string]json.RawMessage{
				"title": mustWidgetRawJSON(t, "Shown"),
			},
		},
		{
			Type:    "intro",
			Enabled: &disabled,
			Props: map[string]json.RawMessage{
				"title": mustWidgetRawJSON(t, "Hidden"),
			},
		},
	}
	out, _, err := renderWidgetTree(ctx, widgets)
	if err != nil {
		t.Fatalf("renderWidgetTree failed: %v", err)
	}
	if !strings.Contains(string(out), "Shown") {
		t.Fatalf("expected enabled intro: %s", string(out))
	}
	if strings.Contains(string(out), "Hidden") {
		t.Fatalf("disabled widget should not render: %s", string(out))
	}
}

func TestRenderWidgetTreeRecognizesMediaSwiper(t *testing.T) {
	ctx := testRenderCtx(t, "content/demo/pages/home.json")
	widgets := []WidgetNode{
		{
			Type: "media_swiper",
			Props: map[string]json.RawMessage{
				"images": mustWidgetRawJSON(t, []map[string]string{{"src": "assets/pic1.png"}}),
			},
		},
	}
	out, _, err := renderWidgetTree(ctx, widgets)
	if err != nil {
		t.Fatalf("renderWidgetTree failed: %v", err)
	}
	html := string(out)
	if !strings.Contains(html, `data-widget-type="media_swiper"`) {
		t.Fatalf("expected media_swiper marker: %s", html)
	}
	if !strings.Contains(html, `data-catalog-carousel`) || !strings.Contains(html, `class="catalog-carousel__slide`) {
		t.Fatalf("expected catalog-carousel-compatible markup: %s", html)
	}
}

func TestProjectGridRendersCards(t *testing.T) {
	ctx := testRenderCtx(t, "content/demo/pages/home.json")
	widgets := []WidgetNode{
		{
			Type: "project_grid",
			Props: map[string]json.RawMessage{
				"heading":               mustWidgetRawJSON(t, "Projects"),
				"subheading":            mustWidgetRawJSON(t, "Selected work"),
				"section_id":            mustWidgetRawJSON(t, "portfolio"),
				"min_card_column_width": mustWidgetRawJSON(t, "280px"),
				"cards": mustWidgetRawJSON(t, []map[string]any{
					{
						"title":       "Alpha",
						"description": "Short description.",
						"tags":        []string{"Go", "Web"},
						"image":       "assets/pic1.png",
						"meta":        map[string]string{"year": "2024", "platform": "Web"},
						"cta":         map[string]string{"label": "Open", "url": "https://example.com/p"},
					},
				}),
			},
		},
	}
	out, _, err := renderWidgetTree(ctx, widgets)
	if err != nil {
		t.Fatalf("renderWidgetTree failed: %v", err)
	}
	html := string(out)
	for _, needle := range []string{`data-widget-type="project_grid"`, `id="portfolio"`, `--project-grid-min:280px`, `project-card__tags`, `<dt>platform</dt>`, `<dt>year</dt>`, `project-card__cta`} {
		if !strings.Contains(html, needle) {
			t.Fatalf("expected %q in output, got: %s", needle, html)
		}
	}
}

func TestProjectGridOptionalImage(t *testing.T) {
	ctx := testRenderCtx(t, "content/demo/pages/home.json")
	widgets := []WidgetNode{
		{
			Type: "project_grid",
			Props: map[string]json.RawMessage{
				"cards": mustWidgetRawJSON(t, []map[string]any{
					{
						"title":       "Text only",
						"description": "No thumbnail.",
						"tags":        []string{"Go"},
						"meta":        "Sample",
						"cta":         map[string]string{"url": "https://example.com/t"},
					},
				}),
			},
		},
	}
	out, _, err := renderWidgetTree(ctx, widgets)
	if err != nil {
		t.Fatalf("renderWidgetTree failed: %v", err)
	}
	html := string(out)
	if strings.Contains(html, "project-card__media") {
		t.Fatalf("expected no media block: %s", html)
	}
	if !strings.Contains(html, "project-card--no-media") {
		t.Fatalf("expected no-media modifier: %s", html)
	}
}

func TestProjectGridMetaString(t *testing.T) {
	ctx := testRenderCtx(t, "content/demo/pages/home.json")
	widgets := []WidgetNode{
		{
			Type: "project_grid",
			Props: map[string]json.RawMessage{
				"cards": mustWidgetRawJSON(t, []map[string]any{
					{
						"title":       "Beta",
						"description": "Desc",
						"tags":        []string{},
						"image":       "assets/pic2.png",
						"meta":        "Highlighted release",
						"cta":         map[string]string{"url": "#intro_title"},
					},
				}),
			},
		},
	}
	out, _, err := renderWidgetTree(ctx, widgets)
	if err != nil {
		t.Fatalf("renderWidgetTree failed: %v", err)
	}
	if !strings.Contains(string(out), "Highlighted release") {
		t.Fatalf("expected meta line: %s", string(out))
	}
}

func TestImagesGridWarnsOnGenericAltFallback(t *testing.T) {
	ctx := testRenderCtx(t, "content/demo/pages/home.json")
	widgets := []WidgetNode{
		{
			Type: "images_grid",
			Props: map[string]json.RawMessage{
				"images": mustWidgetRawJSON(t, []string{"assets/pic1.png"}),
			},
		},
	}
	_, warnings, err := renderWidgetTree(ctx, widgets)
	if err != nil {
		t.Fatalf("renderWidgetTree failed: %v", err)
	}
	if len(warnings) != 1 {
		t.Fatalf("expected 1 warning, got %d: %v", len(warnings), warnings)
	}
	if !strings.Contains(warnings[0].String(), "missing descriptive alt text") {
		t.Fatalf("unexpected warning: %s", warnings[0].String())
	}
}

func mustWidgetRawJSON(t *testing.T, value any) json.RawMessage {
	t.Helper()
	b, err := json.Marshal(value)
	if err != nil {
		t.Fatalf("marshal raw json: %v", err)
	}
	return json.RawMessage(b)
}
