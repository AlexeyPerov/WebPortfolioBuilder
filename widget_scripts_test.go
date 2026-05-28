package main

import (
	"encoding/json"
	"testing"
)

func TestCollectPageScriptNeedsIntroOnly(t *testing.T) {
	needs := collectPageScriptNeeds([]WidgetNode{{Type: "intro"}})
	if !needs.ScrollReveal {
		t.Fatal("intro should require scroll-reveal.js")
	}
	if needs.CatalogCarousel || needs.SplitWidget || needs.ImageLightbox {
		t.Fatalf("intro-only page should not need carousel/split/lightbox scripts: %+v", needs)
	}
}

func TestCollectPageScriptNeedsKometaHomeMix(t *testing.T) {
	needs := collectPageScriptNeeds([]WidgetNode{
		{Type: "intro"},
		{Type: "apps_showcase"},
		{Type: "images_grid"},
		{Type: "careers_tabs"},
	})
	if !needs.ScrollReveal || !needs.CatalogCarousel || !needs.SplitWidget || !needs.ImageLightbox {
		t.Fatalf("expected all interactive scripts for Kometa-like home: %+v", needs)
	}
	if !needs.needsWidgetsConfig() {
		t.Fatal("Kometa-like home should inject site-widgets-config")
	}
}

func TestCollectPageScriptNeedsNestedLayout(t *testing.T) {
	needs := collectPageScriptNeeds([]WidgetNode{{
		Type: "grid",
		Props: map[string]json.RawMessage{
			"children": json.RawMessage(`[{"type":"media_swiper"}]`),
		},
	}})
	if !needs.CatalogCarousel {
		t.Fatal("nested media_swiper should require catalog-carousel.js")
	}
}
