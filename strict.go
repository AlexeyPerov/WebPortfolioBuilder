package main

import (
	"fmt"
	"io"
	"sort"
	"strings"
)

// widgetAllowedPropKeys lists known props keys per widget type (WidgetRegistryV1).
// Unknown props keys warn in default mode and fail when --strict is set.
var widgetAllowedPropKeys = map[string]map[string]struct{}{
	"intro":         keySet("title", "paragraphs"),
	"cover_banner":  keySet("src", "alt"),
	"follow_us":     keySet("title"),
	"info_grid":     keySet("title", "items"),
	"images_grid":   keySet("title", "images"),
	"careers_tabs":  keySet("title", "vacancies", "labels"),
	"apps_showcase": keySet("section_title", "apps"),
	"project_grid":  keySet("heading", "subheading", "section_id", "cards", "min_card_column_width"),
	"media_swiper":  keySet("images", "aria_label"),
	"row":           keySet("children", "gap"),
	"column":        keySet("children"),
	"grid":          keySet("children", "min_column_width", "gap"),
}

func unknownWidgetPropKeyWarning(pagePath, widgetPath, key string) ConfigWarning {
	return contentWarning(pagePath, fmt.Sprintf("%s.props: unknown props key: %q", widgetPath, key))
}

func unknownWidgetPropKeyWarnings(pagePath string, widgets []WidgetNode) []ConfigWarning {
	var warnings []ConfigWarning

	var walk func(pathPrefix string, nodes []WidgetNode)
	walk = func(pathPrefix string, nodes []WidgetNode) {
		for i, w := range nodes {
			wpath := fmt.Sprintf("%s[%d]", pathPrefix, i)
			widgetType := strings.TrimSpace(w.Type)
			allowed, ok := widgetAllowedPropKeys[widgetType]
			if !ok {
				continue
			}
			for key := range w.Props {
				if _, ok := allowed[key]; !ok {
					warnings = append(warnings, unknownWidgetPropKeyWarning(pagePath, wpath, key))
				}
			}
			switch widgetType {
			case "row", "column", "grid":
				children, err := widgetLayoutChildren(w.Props)
				if err != nil || len(children) == 0 {
					continue
				}
				walk(wpath+".props.children", children)
			}
		}
	}

	walk("widgets", widgets)
	return warnings
}

func isStrictEligibleWarning(w ConfigWarning) bool {
	if w.Key != "" {
		return true
	}
	return strings.Contains(w.Detail, "unknown props key:")
}

func enforceStrictWarnings(warnings []ConfigWarning) error {
	var failures []string
	for _, w := range warnings {
		if isStrictEligibleWarning(w) {
			failures = append(failures, w.String())
		}
	}
	sort.Strings(failures)
	if len(failures) == 0 {
		return nil
	}
	if len(failures) == 1 {
		return fmt.Errorf("%s", failures[0])
	}
	return fmt.Errorf("strict validation failed (%d issues):\n  - %s", len(failures), strings.Join(failures, "\n  - "))
}

func handleConfigWarnings(stderr io.Writer, warnings []ConfigWarning, strict bool) error {
	printConfigWarnings(stderr, warnings)
	if strict {
		return enforceStrictWarnings(warnings)
	}
	return nil
}
