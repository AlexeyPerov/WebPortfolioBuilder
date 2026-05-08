package main

import (
	"encoding/json"
	"strings"
	"testing"
)

func TestRenderWidgetTreeUnknownTypeFailsWithPath(t *testing.T) {
	widgets := []WidgetNode{
		{Type: "unknown"},
	}
	_, err := renderWidgetTree("sites/demo/pages/home.json", widgets)
	if err == nil {
		t.Fatal("expected unknown type error")
	}
	if !strings.Contains(err.Error(), "widgets[0].type") {
		t.Fatalf("expected widget path in error, got: %v", err)
	}
}

func TestRenderWidgetTreeRejectsColumnsAlias(t *testing.T) {
	widgets := []WidgetNode{
		{Type: "columns"},
	}
	_, err := renderWidgetTree("sites/demo/pages/home.json", widgets)
	if err == nil {
		t.Fatal("expected columns alias rejection")
	}
	if !strings.Contains(err.Error(), `unknown widget type "columns"`) {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestRenderWidgetTreeLeafChildrenFails(t *testing.T) {
	widgets := []WidgetNode{
		{
			Type: "intro",
			Props: map[string]json.RawMessage{
				"children": mustWidgetRawJSON(t, []WidgetNode{{Type: "intro"}}),
			},
		},
	}
	_, err := renderWidgetTree("sites/demo/pages/home.json", widgets)
	if err == nil {
		t.Fatal("expected leaf children error")
	}
	if !strings.Contains(err.Error(), "only layout widgets") {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestRenderWidgetTreeLayoutNeedsChildren(t *testing.T) {
	widgets := []WidgetNode{
		{Type: "row"},
	}
	_, err := renderWidgetTree("sites/demo/pages/home.json", widgets)
	if err == nil {
		t.Fatal("expected missing children error")
	}
	if !strings.Contains(err.Error(), ".props.children") {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestRenderWidgetTreeLayoutRecurses(t *testing.T) {
	widgets := []WidgetNode{
		{
			Type: "row",
			Props: map[string]json.RawMessage{
				"children": mustWidgetRawJSON(t, []WidgetNode{
					{
						Type: "column",
						Props: map[string]json.RawMessage{
							"children": mustWidgetRawJSON(t, []WidgetNode{
								{Type: "intro", ID: "intro1"},
							}),
						},
					},
				}),
			},
		},
	}
	out, err := renderWidgetTree("sites/demo/pages/home.json", widgets)
	if err != nil {
		t.Fatalf("renderWidgetTree failed: %v", err)
	}
	html := string(out)
	if !strings.Contains(html, `widget-layout--row`) {
		t.Fatalf("expected row layout wrapper, got: %s", html)
	}
	if !strings.Contains(html, `widget-layout--column`) {
		t.Fatalf("expected column layout wrapper, got: %s", html)
	}
	if !strings.Contains(html, `widget-leaf--intro`) {
		t.Fatalf("expected intro leaf widget, got: %s", html)
	}
}

func TestRenderWidgetTreeSkipsDisabledWidgets(t *testing.T) {
	disabled := false
	widgets := []WidgetNode{
		{Type: "intro", Enabled: &disabled},
	}
	out, err := renderWidgetTree("sites/demo/pages/home.json", widgets)
	if err != nil {
		t.Fatalf("renderWidgetTree failed: %v", err)
	}
	if strings.TrimSpace(string(out)) != "" {
		t.Fatalf("expected empty output for disabled widget, got: %s", string(out))
	}
}

func TestRenderWidgetTreeRecognizesMediaSwiper(t *testing.T) {
	widgets := []WidgetNode{
		{Type: "media_swiper"},
	}
	out, err := renderWidgetTree("sites/demo/pages/home.json", widgets)
	if err != nil {
		t.Fatalf("expected media_swiper recognized, got error: %v", err)
	}
	if !strings.Contains(string(out), `widget-leaf--media_swiper`) {
		t.Fatalf("expected media_swiper leaf output, got: %s", string(out))
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
