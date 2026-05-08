package main

import (
	"encoding/json"
	"fmt"
	"html/template"
	"sort"
	"strings"
)

var layoutWidgetTypes = map[string]struct{}{
	"row":    {},
	"column": {},
	"grid":   {},
}

var leafWidgetTypes = map[string]struct{}{
	"intro":         {},
	"apps_showcase": {},
	"info_grid":     {},
	"images_grid":   {},
	"careers_tabs":  {},
	"follow_us":     {},
	"cover_banner":  {},
	"project_grid":  {},
	"media_swiper":  {},
}

func renderWidgetTree(pagePath string, widgets []WidgetNode) (template.HTML, error) {
	var b strings.Builder
	for i, widget := range widgets {
		path := fmt.Sprintf("%s -> widgets[%d]", pagePath, i)
		html, err := renderWidget(path, widget)
		if err != nil {
			return "", err
		}
		if html == "" {
			continue
		}
		if b.Len() > 0 {
			b.WriteString("\n")
		}
		b.WriteString(html)
	}
	return template.HTML(b.String()), nil
}

func renderWidget(path string, widget WidgetNode) (string, error) {
	if widget.Enabled != nil && !*widget.Enabled {
		return "", nil
	}

	widgetType := strings.TrimSpace(widget.Type)
	if widgetType == "" {
		return "", fmt.Errorf("%s.type: required field missing", path)
	}
	if widgetType == "columns" {
		return "", fmt.Errorf("%s.type: unknown widget type %q", path, widgetType)
	}
	if _, ok := layoutWidgetTypes[widgetType]; ok {
		return renderLayoutWidget(path, widgetType, widget)
	}
	if _, ok := leafWidgetTypes[widgetType]; ok {
		if _, hasChildren := widget.Props["children"]; hasChildren {
			return "", fmt.Errorf("%s.props.children: only layout widgets (row, column, grid) may define children", path)
		}
		return renderLeafWidget(path, widgetType, widget), nil
	}

	return "", fmt.Errorf("%s.type: unknown widget type %q", path, widgetType)
}

func renderLayoutWidget(path, widgetType string, widget WidgetNode) (string, error) {
	childrenRaw, ok := widget.Props["children"]
	if !ok {
		return "", fmt.Errorf("%s.props.children: required for layout widget %q", path, widgetType)
	}
	var children []WidgetNode
	if err := json.Unmarshal(childrenRaw, &children); err != nil {
		return "", fmt.Errorf("%s.props.children: invalid children array: %w", path, err)
	}
	if len(children) == 0 {
		return "", fmt.Errorf("%s.props.children: must not be empty for layout widget %q", path, widgetType)
	}

	var childHTML strings.Builder
	for i, child := range children {
		childPath := fmt.Sprintf("%s.props.children[%d]", path, i)
		rendered, err := renderWidget(childPath, child)
		if err != nil {
			return "", err
		}
		if rendered == "" {
			continue
		}
		if childHTML.Len() > 0 {
			childHTML.WriteString("\n")
		}
		childHTML.WriteString(rendered)
	}

	idAttr := ""
	if id := strings.TrimSpace(widget.ID); id != "" {
		idAttr = ` id="` + template.HTMLEscapeString(id) + `"`
	}
	return fmt.Sprintf(
		`<section class="widget widget-layout widget-layout--%s"%s data-widget-type="%s">%s</section>`,
		template.HTMLEscapeString(widgetType),
		idAttr,
		template.HTMLEscapeString(widgetType),
		childHTML.String(),
	), nil
}

func renderLeafWidget(path, widgetType string, widget WidgetNode) string {
	idAttr := ""
	if id := strings.TrimSpace(widget.ID); id != "" {
		idAttr = ` id="` + template.HTMLEscapeString(id) + `"`
	}
	propKeys := make([]string, 0, len(widget.Props))
	for key := range widget.Props {
		if key == "children" {
			continue
		}
		propKeys = append(propKeys, key)
	}
	sort.Strings(propKeys)

	return fmt.Sprintf(
		`<section class="widget widget-leaf widget-leaf--%s"%s data-widget-type="%s" data-widget-path="%s" data-props-count="%d"></section>`,
		template.HTMLEscapeString(widgetType),
		idAttr,
		template.HTMLEscapeString(widgetType),
		template.HTMLEscapeString(path),
		len(propKeys),
	)
}
