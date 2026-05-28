package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"html"
	"html/template"
	"os"
	"sort"
	"strings"
	"unicode"
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

type widgetRenderContext struct {
	PagePath  string
	Site      SiteConfig
	Route     PageRoute
	Routes    RouteIndex
	WidgetTpl *template.Template
}

func renderWidgetTree(ctx *widgetRenderContext, widgets []WidgetNode) (template.HTML, []ConfigWarning, error) {
	var b strings.Builder
	var warnings []ConfigWarning
	for i, widget := range widgets {
		path := fmt.Sprintf("%s -> widgets[%d]", ctx.PagePath, i)
		htmlStr, widgetWarnings, err := renderWidget(ctx, path, widget)
		if err != nil {
			return "", warnings, err
		}
		warnings = append(warnings, widgetWarnings...)
		if htmlStr == "" {
			continue
		}
		if b.Len() > 0 {
			b.WriteString("\n")
		}
		b.WriteString(htmlStr)
	}
	return template.HTML(b.String()), warnings, nil
}

func renderWidget(ctx *widgetRenderContext, path string, widget WidgetNode) (string, []ConfigWarning, error) {
	if widget.Enabled != nil && !*widget.Enabled {
		return "", nil, nil
	}

	widgetType := strings.TrimSpace(widget.Type)
	if widgetType == "" {
		return "", nil, fmt.Errorf("%s.type: required field missing", path)
	}
	if widgetType == "columns" {
		return "", nil, fmt.Errorf("%s.type: unknown widget type %q", path, widgetType)
	}
	if _, ok := layoutWidgetTypes[widgetType]; ok {
		return renderLayoutWidget(ctx, path, widgetType, widget)
	}
	if _, ok := leafWidgetTypes[widgetType]; ok {
		if _, hasChildren := widget.Props["children"]; hasChildren {
			return "", nil, fmt.Errorf("%s.props.children: only layout widgets (row, column, grid) may define children", path)
		}
		return renderLeafWidget(ctx, path, widgetType, widget)
	}

	return "", nil, fmt.Errorf("%s.type: unknown widget type %q", path, widgetType)
}

func renderLayoutWidget(ctx *widgetRenderContext, path, widgetType string, widget WidgetNode) (string, []ConfigWarning, error) {
	childrenRaw, ok := widget.Props["children"]
	if !ok {
		return "", nil, fmt.Errorf("%s.props.children: required for layout widget %q", path, widgetType)
	}
	var children []WidgetNode
	if err := json.Unmarshal(childrenRaw, &children); err != nil {
		return "", nil, fmt.Errorf("%s.props.children: invalid children array: %w", path, err)
	}
	if len(children) == 0 {
		return "", nil, fmt.Errorf("%s.props.children: must not be empty for layout widget %q", path, widgetType)
	}

	var childHTML strings.Builder
	var warnings []ConfigWarning
	for i, child := range children {
		childPath := fmt.Sprintf("%s.props.children[%d]", path, i)
		rendered, childWarnings, err := renderWidget(ctx, childPath, child)
		if err != nil {
			return "", warnings, err
		}
		warnings = append(warnings, childWarnings...)
		if rendered == "" {
			continue
		}
		if childHTML.Len() > 0 {
			childHTML.WriteString("\n")
		}
		childHTML.WriteString(rendered)
	}
	if childHTML.Len() == 0 {
		return "", warnings, fmt.Errorf("%s.props.children: rendered content must not be empty for layout widget %q", path, widgetType)
	}

	childrenOut := template.HTML(childHTML.String())
	widgetID := strings.TrimSpace(widget.ID)
	if widgetID != "" && !isSafeHTMLID(widgetID) {
		return "", warnings, fmt.Errorf("%s.id: invalid id %q", path, widgetID)
	}

	switch widgetType {
	case "row":
		gapRaw, gapErr := readOptionalGap(widget.Props, path+".props.gap")
		if gapErr != nil {
			return "", warnings, gapErr
		}
		var rowStyle template.CSS
		if gapRaw != "" {
			rowStyle = template.CSS(fmt.Sprintf("gap:%s;", gapRaw))
		}
		out, err := executeLayoutTemplate(ctx.WidgetTpl, "row", map[string]any{
			"ID":       widgetID,
			"Children": childrenOut,
			"Style":    rowStyle,
		})
		return out, warnings, err

	case "column":
		gapRaw, gapErr := readOptionalGap(widget.Props, path+".props.gap")
		if gapErr != nil {
			return "", warnings, gapErr
		}
		var colStyle template.CSS
		if gapRaw != "" {
			colStyle = template.CSS(fmt.Sprintf("gap:%s;", gapRaw))
		}
		out, err := executeLayoutTemplate(ctx.WidgetTpl, "column", map[string]any{
			"ID":       widgetID,
			"Children": childrenOut,
			"Style":    colStyle,
		})
		return out, warnings, err

	case "grid":
		minCW, err := parseGridMinColumnWidth(widget.Props, path+".props.min_column_width")
		if err != nil {
			return "", warnings, err
		}
		gapRaw, gapErr := readOptionalGap(widget.Props, path+".props.gap")
		if gapErr != nil {
			return "", warnings, gapErr
		}
		gridGap := "1.25rem"
		if gapRaw != "" {
			gridGap = gapRaw
		}
		innerCSS := template.CSS(fmt.Sprintf("--widget-grid-min: %s; gap: %s;", minCW, gridGap))
		out, err := executeLayoutTemplate(ctx.WidgetTpl, "grid", map[string]any{
			"ID":         widgetID,
			"Children":   childrenOut,
			"InnerStyle": innerCSS,
		})
		return out, warnings, err
	default:
		return "", warnings, fmt.Errorf("%s.type: unsupported layout widget %q", path, widgetType)
	}
}

func executeLayoutTemplate(tpl *template.Template, name string, data any) (string, error) {
	var buf bytes.Buffer
	if err := tpl.ExecuteTemplate(&buf, name, data); err != nil {
		return "", fmt.Errorf("cannot execute widget template %q: %w", name, err)
	}
	return buf.String(), nil
}

func parseGridMinColumnWidth(props map[string]json.RawMessage, cfgPath string) (string, error) {
	const defaultMin = "260px"
	raw, ok := props["min_column_width"]
	if !ok {
		return defaultMin, nil
	}
	var s string
	if err := json.Unmarshal(raw, &s); err != nil {
		return "", fmt.Errorf(`%s: must be a string`, cfgPath)
	}
	s = strings.TrimSpace(s)
	if s == "" {
		return defaultMin, nil
	}
	if strings.Contains(strings.ToLower(s), "..") {
		return "", fmt.Errorf(`%s: invalid value %q`, cfgPath, s)
	}
	safe, err := sanitizeCSSGapOrLength(s, cfgPath)
	if err != nil {
		return "", err
	}
	return safe, nil
}

func readOptionalGap(props map[string]json.RawMessage, cfgPath string) (string, error) {
	raw, ok := props["gap"]
	if !ok {
		return "", nil
	}
	var s string
	if err := json.Unmarshal(raw, &s); err != nil {
		return "", fmt.Errorf("%s: must be a string", cfgPath)
	}
	return sanitizeCSSGapOrLength(strings.TrimSpace(s), cfgPath)
}

func sanitizeCSSGapOrLength(s, cfgPath string) (string, error) {
	if s == "" {
		return "", nil
	}
	if len(s) > 40 {
		return "", fmt.Errorf("%s: value too long", cfgPath)
	}
	for _, r := range s {
		ok := unicode.IsDigit(r) || unicode.IsLetter(r)
		ok = ok || r == '%' || r == '.' || r == '-' || r == '_' ||
			r == '/' || r == ',' || r == '(' || r == ')' || r == '#' || unicode.IsSpace(r)
		if !ok {
			return "", fmt.Errorf("%s: unsupported character in %q", cfgPath, s)
		}
	}
	return s, nil
}

func isSafeHTMLID(id string) bool {
	for i, r := range id {
		if i == 0 {
			if r != '_' && !unicode.IsLetter(r) && !unicode.IsDigit(r) {
				return false
			}
			continue
		}
		if r != '_' && r != '-' && !unicode.IsLetter(r) && !unicode.IsDigit(r) {
			return false
		}
	}
	return true
}

func renderLeafWidget(ctx *widgetRenderContext, path, widgetType string, widget WidgetNode) (string, []ConfigWarning, error) {
	var buf bytes.Buffer
	var warnings []ConfigWarning
	switch widgetType {
	case "intro":
		data, err := parseIntroProps(ctx, widget, path)
		if err != nil {
			return "", nil, err
		}
		if data == nil {
			return "", nil, nil
		}
		if err := ctx.WidgetTpl.ExecuteTemplate(&buf, "intro", data); err != nil {
			return "", nil, fmt.Errorf("cannot render intro: %w", err)
		}
	case "cover_banner":
		data, err := parseCoverBannerProps(ctx, widget, path)
		if err != nil {
			return "", nil, err
		}
		if err := ctx.WidgetTpl.ExecuteTemplate(&buf, "cover_banner", data); err != nil {
			return "", nil, fmt.Errorf("cannot render cover_banner: %w", err)
		}
	case "follow_us":
		data, err := buildFollowUsData(ctx, widget, path)
		if err != nil {
			return "", nil, err
		}
		if data == nil {
			return "", nil, nil
		}
		if err := ctx.WidgetTpl.ExecuteTemplate(&buf, "follow_us", data); err != nil {
			return "", nil, fmt.Errorf("cannot render follow_us: %w", err)
		}
	case "info_grid":
		data, err := parseInfoGridProps(ctx, widget, path)
		if err != nil {
			return "", nil, err
		}
		if err := ctx.WidgetTpl.ExecuteTemplate(&buf, "info_grid", data); err != nil {
			return "", nil, fmt.Errorf("cannot render info_grid: %w", err)
		}
	case "images_grid":
		data, imageWarnings, err := parseImagesGridProps(ctx, widget, path)
		if err != nil {
			return "", imageWarnings, err
		}
		warnings = append(warnings, imageWarnings...)
		if err := ctx.WidgetTpl.ExecuteTemplate(&buf, "images_grid", data); err != nil {
			return "", warnings, fmt.Errorf("cannot render images_grid: %w", err)
		}
	case "careers_tabs":
		data, err := parseCareersTabsProps(ctx, widget, path)
		if err != nil {
			return "", nil, err
		}
		if err := ctx.WidgetTpl.ExecuteTemplate(&buf, "careers_tabs", data); err != nil {
			return "", nil, fmt.Errorf("cannot render careers_tabs: %w", err)
		}
	case "apps_showcase":
		id := strings.TrimSpace(widget.ID)
		if id != "" && !isSafeHTMLID(id) {
			return "", nil, fmt.Errorf("%s.id: invalid id %q", path, id)
		}
		data, storeWarnings, err := parseAppsShowcaseProps(ctx, widget, path, id)
		if err != nil {
			return "", nil, err
		}
		warnings = append(warnings, storeWarnings...)
		if err := ctx.WidgetTpl.ExecuteTemplate(&buf, "apps_showcase", data); err != nil {
			return "", nil, fmt.Errorf("cannot render apps_showcase: %w", err)
		}
	case "project_grid":
		id := strings.TrimSpace(widget.ID)
		if id != "" && !isSafeHTMLID(id) {
			return "", nil, fmt.Errorf("%s.id: invalid id %q", path, id)
		}
		data, err := parseProjectGridProps(ctx, widget, path)
		if err != nil {
			return "", nil, err
		}
		if err := ctx.WidgetTpl.ExecuteTemplate(&buf, "project_grid", data); err != nil {
			return "", nil, fmt.Errorf("cannot render project_grid: %w", err)
		}
	case "media_swiper":
		id := strings.TrimSpace(widget.ID)
		if id != "" && !isSafeHTMLID(id) {
			return "", nil, fmt.Errorf("%s.id: invalid id %q", path, id)
		}
		data, err := parseMediaSwiperProps(ctx, widget, path)
		if err != nil {
			return "", nil, err
		}
		if err := ctx.WidgetTpl.ExecuteTemplate(&buf, "media_swiper", data); err != nil {
			return "", nil, fmt.Errorf("cannot render media_swiper: %w", err)
		}
	default:
		return "", nil, fmt.Errorf("%s.type: unsupported leaf widget %q", path, widgetType)
	}
	return buf.String(), warnings, nil
}

// --- Intro

type introTemplateData struct {
	Title      string
	Paragraphs []string
}

func parseIntroProps(_ *widgetRenderContext, widget WidgetNode, path string) (*introTemplateData, error) {
	type raw struct {
		Title      string   `json:"title"`
		Paragraphs []string `json:"paragraphs"`
	}
	var p raw
	if len(widget.Props) > 0 {
		b, err := json.Marshal(widget.Props)
		if err != nil {
			return nil, fmt.Errorf("%s.props: invalid json: %w", path, err)
		}
		if err := json.Unmarshal(b, &p); err != nil {
			return nil, fmt.Errorf("%s.props: invalid shape: %w", path, err)
		}
	}
	title := strings.TrimSpace(p.Title)
	var paras []string
	for _, line := range p.Paragraphs {
		line = strings.TrimSpace(line)
		if line != "" {
			paras = append(paras, line)
		}
	}
	if title == "" && len(paras) == 0 {
		fmt.Fprintf(os.Stderr, "Warning: %s -> intro: empty title and paragraphs; rendering nothing\n", path)
		return nil, nil
	}
	return &introTemplateData{Title: title, Paragraphs: paras}, nil
}

// --- Cover banner

type coverBannerData struct {
	Src string
	Alt string
}

func parseCoverBannerProps(ctx *widgetRenderContext, widget WidgetNode, path string) (*coverBannerData, error) {
	type raw struct {
		Src string `json:"src"`
		Alt string `json:"alt"`
	}
	var p raw
	b, err := json.Marshal(widget.Props)
	if err != nil {
		return nil, fmt.Errorf("%s.props: invalid json: %w", path, err)
	}
	if err := json.Unmarshal(b, &p); err != nil {
		return nil, fmt.Errorf("%s.props: invalid shape: %w", path, err)
	}
	src := strings.TrimSpace(p.Src)
	if src == "" {
		return nil, fmt.Errorf("%s.props.src: required field missing", path)
	}
	href := resolveAssetHrefForPage(src, ctx.Route)
	alt := strings.TrimSpace(p.Alt)
	if alt == "" {
		alt = "Cover"
	}
	return &coverBannerData{Src: href, Alt: alt}, nil
}

type followUsTemplateData struct {
	Title      string
	HasButtons bool
	Buttons    template.HTML
}

func buildFollowUsData(ctx *widgetRenderContext, widget WidgetNode, path string) (*followUsTemplateData, error) {
	type raw struct {
		Title string `json:"title"`
	}
	var r raw
	if len(widget.Props) > 0 {
		b, err := json.Marshal(widget.Props)
		if err != nil {
			return nil, fmt.Errorf("%s.props: invalid json: %w", path, err)
		}
		if err := json.Unmarshal(b, &r); err != nil {
			return nil, fmt.Errorf("%s.props: invalid shape: %w", path, err)
		}
	}
	title := strings.TrimSpace(r.Title)
	if title == "" {
		title = "Follow us"
	}

	var buttons []string
	for _, link := range ctx.Site.Social.resolvedSocialLinks() {
		u := strings.TrimSpace(link.URL)
		if u == "" {
			continue
		}
		imgPath := strings.TrimSpace(link.IconImage)
		presetSVG := socialIconPresetSVG(link.Icon)
		if imgPath == "" && presetSVG == "" {
			continue
		}
		aria := strings.TrimSpace(link.AriaLabel)
		if aria == "" {
			if imgPath != "" && strings.TrimSpace(link.Icon) != "" {
				aria = ariaDefaultForSocialPreset(link.Icon)
			} else if presetSVG != "" {
				aria = ariaDefaultForSocialPreset(link.Icon)
			} else {
				aria = "Link"
			}
		}
		var inner string
		btnClass := "follow-us__btn"
		if imgPath != "" {
			h := resolveAssetHrefForPage(imgPath, ctx.Route)
			inner = fmt.Sprintf(`<img class="follow-us__icon" src="%s" alt="" decoding="async">`, html.EscapeString(h))
			btnClass += " follow-us__btn--custom"
		} else {
			inner = presetSVG
			if mod := socialIconPresetClass(link.Icon); mod != "" {
				btnClass += " " + mod
			}
		}
		buttons = append(buttons, fmt.Sprintf(
			`<a class="%s" href="%s"%s aria-label="%s">%s</a>`,
			btnClass,
			html.EscapeString(u),
			externalLinkAttrs(u),
			html.EscapeString(aria),
			inner))
	}

	if len(buttons) == 0 {
		fmt.Fprintf(os.Stderr, "Warning: %s -> follow_us: no social links resolved; rendering nothing\n", path)
		return nil, nil
	}
	bt := strings.Join(buttons, "")
	return &followUsTemplateData{
		Title:      title,
		HasButtons: true,
		Buttons:    template.HTML(bt),
	}, nil
}

type infoGridItemData struct {
	Image    string
	ImageAlt string
	Title    string
	Text     string
}

type infoGridTemplateData struct {
	Title string
	Items []infoGridItemData
}

func parseInfoGridProps(ctx *widgetRenderContext, widget WidgetNode, path string) (*infoGridTemplateData, error) {
	type itemRaw struct {
		Image string `json:"image"`
		Title string `json:"title"`
		Text  string `json:"text"`
	}
	type rawTop struct {
		Title string    `json:"title"`
		Items []itemRaw `json:"items"`
	}
	var p rawTop
	b, err := json.Marshal(widget.Props)
	if err != nil {
		return nil, fmt.Errorf("%s.props: invalid json: %w", path, err)
	}
	if err := json.Unmarshal(b, &p); err != nil {
		return nil, fmt.Errorf("%s.props: invalid shape: %w", path, err)
	}
	if len(p.Items) == 0 {
		return nil, fmt.Errorf("%s.props.items: required and must not be empty", path)
	}
	items := make([]infoGridItemData, 0, len(p.Items))
	for i, it := range p.Items {
		img := ""
		title := strings.TrimSpace(it.Title)
		text := strings.TrimSpace(it.Text)
		imgRaw := strings.TrimSpace(it.Image)
		if imgRaw != "" {
			img = resolveAssetHrefForPage(imgRaw, ctx.Route)
		}
		imgAlt := title
		if imgAlt == "" {
			imgAlt = " "
		}
		if title == "" && text == "" {
			return nil, fmt.Errorf("%s.props.items[%d]: at least one of title or text is required", path, i)
		}
		items = append(items, infoGridItemData{Image: img, ImageAlt: imgAlt, Title: title, Text: text})
	}
	return &infoGridTemplateData{Title: strings.TrimSpace(p.Title), Items: items}, nil
}

type imagesGridEntry struct {
	Src string
	Alt string
}

type imagesGridTemplateData struct {
	Title  string
	Images []imagesGridEntry
}

func parseImagesGridProps(ctx *widgetRenderContext, widget WidgetNode, path string) (*imagesGridTemplateData, []ConfigWarning, error) {
	raw, ok := widget.Props["images"]
	if !ok {
		return nil, nil, fmt.Errorf("%s.props.images: required and must not be empty", path)
	}
	entries, warnings, err := normalizeImagesGridRaw(ctx.PagePath, raw, path+".props.images")
	if err != nil {
		return nil, warnings, err
	}
	if len(entries) == 0 {
		return nil, warnings, fmt.Errorf("%s.props.images: required and must not be empty", path)
	}
	for i := range entries {
		entries[i].Src = resolveAssetHrefForPage(entries[i].Src, ctx.Route)
	}
	type rawTop struct {
		Title string `json:"title"`
	}
	var top rawTop
	if b, err := json.Marshal(widget.Props); err == nil {
		_ = json.Unmarshal(b, &top)
	}
	return &imagesGridTemplateData{Title: strings.TrimSpace(top.Title), Images: entries}, warnings, nil
}

func isGenericImagesGridAlt(alt string, index int) bool {
	alt = strings.TrimSpace(alt)
	if alt == "" {
		return true
	}
	return alt == fmt.Sprintf("photo %d", index+1)
}

func normalizeImagesGridRaw(pagePath string, raw json.RawMessage, cfgPath string) ([]imagesGridEntry, []ConfigWarning, error) {
	var asStrings []string
	if err := json.Unmarshal(raw, &asStrings); err == nil {
		var out []imagesGridEntry
		var warnings []ConfigWarning
		for i, s := range asStrings {
			s = strings.TrimSpace(s)
			if s == "" {
				return nil, warnings, fmt.Errorf("%s[%d]: empty string not allowed", cfgPath, i)
			}
			alt := fmt.Sprintf("photo %d", i+1)
			out = append(out, imagesGridEntry{Src: s, Alt: alt})
			warnings = append(warnings, contentWarning(pagePath,
				fmt.Sprintf("%s[%d]: missing descriptive alt text (using generic fallback %q)", cfgPath, i, alt)))
		}
		return out, warnings, nil
	}
	var asObjs []struct {
		Src string `json:"src"`
		Alt string `json:"alt"`
	}
	if err := json.Unmarshal(raw, &asObjs); err != nil {
		return nil, nil, fmt.Errorf("%s: must be an array of strings or objects with src", cfgPath)
	}
	var out []imagesGridEntry
	var warnings []ConfigWarning
	for i, o := range asObjs {
		src := strings.TrimSpace(o.Src)
		if src == "" {
			return nil, warnings, fmt.Errorf("%s[%d].src: required", cfgPath, i)
		}
		alt := strings.TrimSpace(o.Alt)
		if isGenericImagesGridAlt(alt, i) {
			if alt == "" {
				alt = fmt.Sprintf("photo %d", i+1)
			}
			warnings = append(warnings, contentWarning(pagePath,
				fmt.Sprintf("%s[%d].alt: missing descriptive alt text (using generic fallback %q)", cfgPath, i, alt)))
		}
		out = append(out, imagesGridEntry{Src: src, Alt: alt})
	}
	return out, warnings, nil
}

type appsShowcaseTemplateData struct {
	ID           string
	SectionTitle string
	Cards        []appsShowcaseCardData
}

type appsShowcaseCardData struct {
	CardClass     string
	CardStyleAttr template.HTMLAttr
	HeaderImage   string
	HeaderAlt     string
	TitleInHeader string
	BodyTitle     string
	IconSrc       string
	IconAlt       string
	StatLeft1     string
	StatLeft2     string
	StatRight1    string
	StatRight2    string
	Text1         string
	Text2         string
	Slides        []appsShowcaseSlideData
	Stores        []appsShowcaseStoreData
	Subscribe     *appsShowcaseSubscribeData
}

type appsShowcaseSlideData struct {
	Src string
	Alt string
}

type appsShowcaseStoreData struct {
	ClassSuffix string
	URL         string
	Attrs       template.HTMLAttr
	AriaLabel   string
	IconSrc     string
}

type appsShowcaseSubscribeData struct {
	Title string
	Links []appsShowcaseSubscribeLinkData
}

type appsShowcaseSubscribeLinkData struct {
	URL   string
	Attrs template.HTMLAttr
	Label string
}

func parseAppsShowcaseProps(ctx *widgetRenderContext, widget WidgetNode, path, widgetID string) (*appsShowcaseTemplateData, []ConfigWarning, error) {
	type raw struct {
		SectionTitle string            `json:"section_title"`
		Apps         []json.RawMessage `json:"apps"`
	}
	var p raw
	b, err := json.Marshal(widget.Props)
	if err != nil {
		return nil, nil, fmt.Errorf("%s.props: invalid json: %w", path, err)
	}
	if err := json.Unmarshal(b, &p); err != nil {
		return nil, nil, fmt.Errorf("%s.props: invalid shape: %w", path, err)
	}
	if len(p.Apps) == 0 {
		return nil, nil, fmt.Errorf("%s.props.apps: required and must not be empty", path)
	}

	icons := ctx.Site.StoreIcons
	subscribe := buildAppsShowcaseSubscribeData(ctx.Site.SubscribeBlock)
	cards := make([]appsShowcaseCardData, 0, len(p.Apps))
	var warnings []ConfigWarning
	for i, appRaw := range p.Apps {
		appPath := fmt.Sprintf("%s.props.apps[%d]", path, i)
		var app CatalogApp
		if err := json.Unmarshal(appRaw, &app); err != nil {
			return nil, warnings, fmt.Errorf("%s: invalid app entry: %w", appPath, err)
		}
		warnings = append(warnings, catalogStoreURLWarnings(ctx.PagePath, appPath, appRaw)...)
		card, err := buildAppsShowcaseCardData(ctx, app, appPath, icons, subscribe)
		if err != nil {
			return nil, warnings, err
		}
		cards = append(cards, card)
	}

	return &appsShowcaseTemplateData{
		ID:           widgetID,
		SectionTitle: strings.TrimSpace(p.SectionTitle),
		Cards:        cards,
	}, warnings, nil
}

func buildAppsShowcaseCardData(
	ctx *widgetRenderContext,
	app CatalogApp,
	appPath string,
	icons StoreIcons,
	subscribe *appsShowcaseSubscribeData,
) (appsShowcaseCardData, error) {
	image := strings.TrimSpace(app.Image)
	if image == "" {
		return appsShowcaseCardData{}, fmt.Errorf("%s.image: required field missing", appPath)
	}
	title := strings.TrimSpace(app.Title)
	headerImage := strings.TrimSpace(app.HeaderImage)
	headerHref := ""
	headerAlt := ""
	titleInHeader := ""
	bodyTitle := ""
	if headerImage != "" {
		headerHref = resolveAssetHrefForPage(headerImage, ctx.Route)
		headerAlt = title
		if headerAlt == "" {
			headerAlt = "App"
		}
		if title != "" {
			bodyTitle = title
		}
	} else if title != "" {
		titleInHeader = title
	}

	bg := strings.TrimSpace(app.CardBackground)
	if bg == "" {
		bg = "var(--widget-gradient)"
	}
	cardStyleAttr := template.HTMLAttr(fmt.Sprintf(` style="background: %s"`, html.EscapeString(bg)))
	cardClass := "offer-card catalog-app-card scroll-reveal"
	if titleInHeader != "" {
		cardClass += " catalog-app-card--title-in-header"
	}

	slides := make([]appsShowcaseSlideData, 0, len(app.SwiperImages))
	for i, rawSrc := range app.SwiperImages {
		src := strings.TrimSpace(rawSrc)
		if src == "" {
			return appsShowcaseCardData{}, fmt.Errorf("%s.swiper_images[%d]: required field missing", appPath, i)
		}
		slides = append(slides, appsShowcaseSlideData{
			Src: resolveAssetHrefForPage(src, ctx.Route),
			Alt: fmt.Sprintf("%s screenshot %d", title, i+1),
		})
	}

	resolvedStores := resolveCatalogStoreEntries(app, icons)
	stores := make([]appsShowcaseStoreData, 0, len(resolvedStores))
	for _, store := range resolvedStores {
		if strings.TrimSpace(store.URL) == "" {
			continue
		}
		stores = append(stores, appsShowcaseStoreData{
			ClassSuffix: store.ClassSuffix,
			URL:         store.URL,
			Attrs:       template.HTMLAttr(externalLinkAttrs(store.URL)),
			AriaLabel:   store.AriaLabel,
			IconSrc:     resolveAssetHrefForPage(store.IconSrc, ctx.Route),
		})
	}

	return appsShowcaseCardData{
		CardClass:     cardClass,
		CardStyleAttr: cardStyleAttr,
		HeaderImage:   headerHref,
		HeaderAlt:     headerAlt,
		TitleInHeader: titleInHeader,
		BodyTitle:     bodyTitle,
		IconSrc:       resolveAssetHrefForPage(image, ctx.Route),
		IconAlt:       title,
		StatLeft1:     catalogStatLineOr(app.StatLeftLine1, "1M+"),
		StatLeft2:     catalogStatLineOr(app.StatLeftLine2, "Downloads"),
		StatRight1:    catalogStatLineOr(app.StatRightLine1, "4.8"),
		StatRight2:    catalogStatLineOr(app.StatRightLine2, "on Google Play"),
		Text1:         strings.TrimSpace(app.Text1),
		Text2:         strings.TrimSpace(app.Text2),
		Slides:        slides,
		Stores:        stores,
		Subscribe:     subscribe,
	}, nil
}

func buildAppsShowcaseSubscribeData(s SubscribeBlock) *appsShowcaseSubscribeData {
	links := make([]appsShowcaseSubscribeLinkData, 0, len(s.Links))
	for _, link := range s.Links {
		u := strings.TrimSpace(link.URL)
		if u == "" {
			continue
		}
		label := strings.TrimSpace(link.Label)
		if label == "" {
			label = u
		}
		links = append(links, appsShowcaseSubscribeLinkData{
			URL:   u,
			Attrs: template.HTMLAttr(externalLinkAttrs(u)),
			Label: label,
		})
	}
	if len(links) == 0 {
		return nil
	}
	title := strings.TrimSpace(s.Title)
	if title == "" {
		title = "Subscribe for news"
	}
	return &appsShowcaseSubscribeData{
		Title: title,
		Links: links,
	}
}

// --- project_grid

type projectGridTemplateData struct {
	SectionIDAttr template.HTMLAttr
	Heading       string
	Subheading    string
	GridStyle     template.CSS
	Cards         []projectGridCardTemplateData
}

type projectGridCardTemplateData struct {
	ImageSrc     string
	ImageAlt     string
	Title        string
	Description  string
	Tags         []string
	MetaLine     string
	MetaPairs    []projectGridMetaPair
	HasMetaPairs bool
	CtaLabel     string
	CtaURL       string
	CtaAttrs     template.HTMLAttr
}

type projectGridMetaPair struct {
	Key   string
	Value string
}

func parseProjectGridProps(ctx *widgetRenderContext, widget WidgetNode, path string) (*projectGridTemplateData, error) {
	raw, err := json.Marshal(widget.Props)
	if err != nil {
		return nil, fmt.Errorf("%s.props: invalid json: %w", path, err)
	}
	type ctaRaw struct {
		Label string `json:"label"`
		URL   string `json:"url"`
	}
	type cardRaw struct {
		Title       string          `json:"title"`
		Description string          `json:"description"`
		Tags        []string        `json:"tags"`
		Image       string          `json:"image"`
		CTA         ctaRaw          `json:"cta"`
		Meta        json.RawMessage `json:"meta"`
	}
	type topRaw struct {
		Heading            string    `json:"heading"`
		Subheading         string    `json:"subheading"`
		SectionID          string    `json:"section_id"`
		MinCardColumnWidth string    `json:"min_card_column_width"`
		Cards              []cardRaw `json:"cards"`
	}
	var tr topRaw
	if err := json.Unmarshal(raw, &tr); err != nil {
		return nil, fmt.Errorf("%s.props: invalid shape: %w", path, err)
	}
	if len(tr.Cards) == 0 {
		return nil, fmt.Errorf("%s.props.cards: required and must not be empty", path)
	}

	sectionID := strings.TrimSpace(tr.SectionID)
	var sectionAttr template.HTMLAttr
	if sectionID != "" {
		if !isSafeHTMLID(sectionID) {
			return nil, fmt.Errorf("%s.props.section_id: invalid id %q", path, sectionID)
		}
		sectionAttr = template.HTMLAttr(` id="` + html.EscapeString(sectionID) + `"`)
	}

	minW, err := parseMinCardColumnWidth(widget.Props, path+".props.min_card_column_width")
	if err != nil {
		return nil, err
	}
	gridStyle := template.CSS(fmt.Sprintf("--project-grid-min:%s;display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%%,var(--project-grid-min)),1fr));gap:1.25rem;", minW))

	out := &projectGridTemplateData{
		SectionIDAttr: sectionAttr,
		Heading:       strings.TrimSpace(tr.Heading),
		Subheading:    strings.TrimSpace(tr.Subheading),
		GridStyle:     gridStyle,
		Cards:         make([]projectGridCardTemplateData, 0, len(tr.Cards)),
	}

	for i, c := range tr.Cards {
		cardPath := fmt.Sprintf("%s.props.cards[%d]", path, i)
		title := strings.TrimSpace(c.Title)
		if title == "" {
			return nil, fmt.Errorf("%s.title: required field missing", cardPath)
		}
		desc := strings.TrimSpace(c.Description)
		if desc == "" {
			return nil, fmt.Errorf("%s.description: required field missing", cardPath)
		}
		img := strings.TrimSpace(c.Image)
		if img == "" {
			return nil, fmt.Errorf("%s.image: required field missing", cardPath)
		}
		ctaURL := strings.TrimSpace(c.CTA.URL)
		if ctaURL == "" {
			return nil, fmt.Errorf("%s.cta.url: required field missing", cardPath)
		}
		ctaLabel := strings.TrimSpace(c.CTA.Label)
		if ctaLabel == "" {
			ctaLabel = "Learn more"
		}
		resolvedCTA, err := resolveProjectGridCTA(ctx, ctaURL)
		if err != nil {
			return nil, fmt.Errorf("%s.cta.url: %w", cardPath, err)
		}
		metaLine, metaPairs, err := parseProjectGridCardMeta(c.Meta, cardPath+".meta")
		if err != nil {
			return nil, err
		}
		var tags []string
		for _, t := range c.Tags {
			t = strings.TrimSpace(t)
			if t != "" {
				tags = append(tags, t)
			}
		}

		out.Cards = append(out.Cards, projectGridCardTemplateData{
			ImageSrc:     resolveAssetHrefForPage(img, ctx.Route),
			ImageAlt:     title,
			Title:        title,
			Description:  desc,
			Tags:         tags,
			MetaLine:     metaLine,
			MetaPairs:    metaPairs,
			HasMetaPairs: len(metaPairs) > 0,
			CtaLabel:     ctaLabel,
			CtaURL:       resolvedCTA,
			CtaAttrs:     template.HTMLAttr(externalLinkAttrs(resolvedCTA)),
		})
	}
	return out, nil
}

func parseMinCardColumnWidth(props map[string]json.RawMessage, cfgPath string) (string, error) {
	const defaultMin = "260px"
	raw, ok := props["min_card_column_width"]
	if !ok {
		return defaultMin, nil
	}
	var s string
	if err := json.Unmarshal(raw, &s); err != nil {
		return "", fmt.Errorf(`%s: must be a string`, cfgPath)
	}
	s = strings.TrimSpace(s)
	if s == "" {
		return defaultMin, nil
	}
	if strings.Contains(strings.ToLower(s), "..") {
		return "", fmt.Errorf(`%s: invalid value %q`, cfgPath, s)
	}
	return sanitizeCSSGapOrLength(s, cfgPath)
}

func resolveProjectGridCTA(ctx *widgetRenderContext, raw string) (string, error) {
	u := strings.TrimSpace(raw)
	if u == "" {
		return "", nil
	}
	if isExternalOrSpecialHref(u) || strings.HasPrefix(u, "/") {
		return u, nil
	}
	return resolveInternalSlugReference(ctx.Route, u, ctx.Routes.BySlug)
}

func parseProjectGridCardMeta(raw json.RawMessage, metaPath string) (line string, pairs []projectGridMetaPair, err error) {
	if len(raw) == 0 {
		return "", nil, fmt.Errorf("%s: required field missing", metaPath)
	}
	var asString string
	if err := json.Unmarshal(raw, &asString); err == nil {
		s := strings.TrimSpace(asString)
		if s == "" {
			return "", nil, fmt.Errorf("%s: must not be empty", metaPath)
		}
		return s, nil, nil
	}
	var asObj map[string]string
	if err := json.Unmarshal(raw, &asObj); err != nil {
		return "", nil, fmt.Errorf("%s: must be a string or object with string values", metaPath)
	}
	keys := make([]string, 0, len(asObj))
	for k := range asObj {
		keys = append(keys, k)
	}
	sort.Strings(keys)
	if len(keys) == 0 {
		return "", nil, fmt.Errorf("%s: object must not be empty", metaPath)
	}
	for _, k := range keys {
		v := strings.TrimSpace(asObj[k])
		if v == "" {
			return "", nil, fmt.Errorf("%s.%s: value must not be empty", metaPath, k)
		}
		pairs = append(pairs, projectGridMetaPair{Key: k, Value: v})
	}
	return "", pairs, nil
}

// --- media_swiper (reuses catalog-carousel.js DOM: [data-catalog-carousel] + .catalog-carousel__* classes).

type mediaSwiperTemplateData struct {
	AriaLabel string
	Slides    []mediaSwiperSlideData
}

type mediaSwiperSlideData struct {
	Src string
	Alt string
}

func parseMediaSwiperProps(ctx *widgetRenderContext, widget WidgetNode, path string) (*mediaSwiperTemplateData, error) {
	imgRaw, ok := widget.Props["images"]
	if !ok {
		return nil, fmt.Errorf("%s.props.images: required and must not be empty", path)
	}
	var slides []struct {
		Src string `json:"src"`
		Alt string `json:"alt"`
	}
	if err := json.Unmarshal(imgRaw, &slides); err != nil {
		return nil, fmt.Errorf("%s.props.images: invalid array: %w", path, err)
	}
	if len(slides) == 0 {
		return nil, fmt.Errorf("%s.props.images: required and must not be empty", path)
	}
	type top struct {
		AriaLabel string `json:"aria_label"`
	}
	var t top
	if b, err := json.Marshal(widget.Props); err == nil {
		_ = json.Unmarshal(b, &t)
	}
	aria := strings.TrimSpace(t.AriaLabel)
	if aria == "" {
		aria = "Image carousel"
	}
	out := &mediaSwiperTemplateData{AriaLabel: aria}
	for i, s := range slides {
		src := strings.TrimSpace(s.Src)
		if src == "" {
			return nil, fmt.Errorf("%s.props.images[%d].src: required field missing", path, i)
		}
		alt := strings.TrimSpace(s.Alt)
		if alt == "" {
			alt = fmt.Sprintf("slide %d", i+1)
		}
		out.Slides = append(out.Slides, mediaSwiperSlideData{
			Src: resolveAssetHrefForPage(src, ctx.Route),
			Alt: alt,
		})
	}
	return out, nil
}

type careersTabsTemplateData struct {
	SectionTitle string
	SplitWidget  template.HTML
}

func parseCareersTabsProps(ctx *widgetRenderContext, widget WidgetNode, path string) (*careersTabsTemplateData, error) {
	type raw struct {
		Title     string            `json:"title"`
		Vacancies []Vacancy         `json:"vacancies"`
		Labels    map[string]string `json:"labels"`
	}
	var p raw
	b, err := json.Marshal(widget.Props)
	if err != nil {
		return nil, fmt.Errorf("%s.props: invalid json: %w", path, err)
	}
	if err := json.Unmarshal(b, &p); err != nil {
		return nil, fmt.Errorf("%s.props: invalid shape: %w", path, err)
	}
	if len(p.Vacancies) == 0 {
		return nil, fmt.Errorf("%s.props.vacancies: required and must not be empty", path)
	}
	req := "Requirements for this position:"
	resp := "Responsibilities:"
	adv := "Advantages of working with us:"
	if p.Labels != nil {
		if v := strings.TrimSpace(p.Labels["requirements_title"]); v != "" {
			req = v
		}
		if v := strings.TrimSpace(p.Labels["responsibilities_title"]); v != "" {
			resp = v
		}
		if v := strings.TrimSpace(p.Labels["advantages_title"]); v != "" {
			adv = v
		}
	}
	reqE := html.EscapeString(req)
	respE := html.EscapeString(resp)
	advE := html.EscapeString(adv)

	entries := make([]widgetSplitEntry, 0, len(p.Vacancies))
	for i, v := range p.Vacancies {
		if strings.TrimSpace(v.Role) == "" {
			return nil, fmt.Errorf("%s.props.vacancies[%d].role: required field missing", path, i)
		}
		body := renderVacancyPanelHTML(v, reqE, respE, advE)
		entries = append(entries, widgetSplitEntry{Label: strings.TrimSpace(v.Role), Body: body})
	}
	splitHTML := template.HTML(buildCareersSplitWidgetHTML(entries))
	return &careersTabsTemplateData{
		SectionTitle: strings.TrimSpace(p.Title),
		SplitWidget:  splitHTML,
	}, nil
}

func buildCareersSplitWidgetHTML(entries []widgetSplitEntry) string {
	return buildCareersSplitWidget("Open positions", "vacancy", entries)
}

// buildCareersSplitWidget mirrors buildSplitWidget in html.go (split-widget.js contract).
func buildCareersSplitWidget(ariaLabel, idPrefix string, entries []widgetSplitEntry) string {
	if len(entries) == 0 {
		return ""
	}
	if len(entries) == 1 {
		return fmt.Sprintf(
			`<div class="split-widget split-widget--single"><div class="split-widget__panels"><div class="split-widget__panel is-active">%s</div></div></div>`,
			entries[0].Body,
		)
	}

	var list strings.Builder
	var panels strings.Builder
	for i, e := range entries {
		id := fmt.Sprintf("%s-%d", idPrefix, i)
		tabClass := "split-widget__tab"
		panelClass := "split-widget__panel"
		ariaSel := "false"
		if i == 0 {
			tabClass += " is-active"
			panelClass += " is-active"
			ariaSel = "true"
		}
		list.WriteString(fmt.Sprintf(
			`<li><button type="button" class="%s" data-target="%s" role="tab" aria-selected="%s">%s</button></li>`,
			tabClass, id, ariaSel, html.EscapeString(e.Label)))
		panels.WriteString(fmt.Sprintf(
			`<div id="%s" class="%s" role="tabpanel">%s</div>`,
			id, panelClass, e.Body))
	}

	return fmt.Sprintf(
		`<div class="split-widget" data-split-widget aria-label="%s"><div class="split-widget__layout"><nav class="split-widget__nav" aria-label="%s"><ul class="split-widget__list">%s</ul></nav><div class="split-widget__panels">%s</div></div></div>`,
		html.EscapeString(ariaLabel),
		html.EscapeString(ariaLabel),
		list.String(),
		panels.String(),
	)
}

func renderVacancyPanelHTML(v Vacancy, reqTitle, respTitle, advTitle string) string {
	var b strings.Builder
	b.WriteString(`<div class="vacancy-detail">`)
	b.WriteString(fmt.Sprintf(`<h3>%s</h3>`, html.EscapeString(v.Role)))
	b.WriteString(fmt.Sprintf(`<h4>%s</h4><ul>%s</ul>`, reqTitle, buildList(v.Requirements)))
	b.WriteString(fmt.Sprintf(`<h4>%s</h4><ul>%s</ul>`, respTitle, buildList(v.Responsibilities)))
	b.WriteString(fmt.Sprintf(`<h4>%s</h4><ul>%s</ul>`, advTitle, buildList(v.Advantages)))
	if u := strings.TrimSpace(v.ApplyURL); u != "" {
		label := strings.TrimSpace(v.ApplyLabel)
		if label == "" {
			label = "Apply for job"
		}
		b.WriteString(fmt.Sprintf(
			`<p class="vacancy-apply"><a class="vacancy-apply-btn" href="%s">%s</a></p>`,
			html.EscapeString(u), html.EscapeString(label)))
	}
	b.WriteString(`</div>`)
	return b.String()
}

type widgetSplitEntry struct {
	Label string
	Body  string
}
