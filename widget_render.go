package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"html"
	"html/template"
	"os"
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
	WidgetTpl *template.Template
}

func renderWidgetTree(ctx *widgetRenderContext, widgets []WidgetNode) (template.HTML, error) {
	var b strings.Builder
	for i, widget := range widgets {
		path := fmt.Sprintf("%s -> widgets[%d]", ctx.PagePath, i)
		htmlStr, err := renderWidget(ctx, path, widget)
		if err != nil {
			return "", err
		}
		if htmlStr == "" {
			continue
		}
		if b.Len() > 0 {
			b.WriteString("\n")
		}
		b.WriteString(htmlStr)
	}
	return template.HTML(b.String()), nil
}

func renderWidget(ctx *widgetRenderContext, path string, widget WidgetNode) (string, error) {
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
		return renderLayoutWidget(ctx, path, widgetType, widget)
	}
	if _, ok := leafWidgetTypes[widgetType]; ok {
		if _, hasChildren := widget.Props["children"]; hasChildren {
			return "", fmt.Errorf("%s.props.children: only layout widgets (row, column, grid) may define children", path)
		}
		return renderLeafWidget(ctx, path, widgetType, widget)
	}

	return "", fmt.Errorf("%s.type: unknown widget type %q", path, widgetType)
}

func renderLayoutWidget(ctx *widgetRenderContext, path, widgetType string, widget WidgetNode) (string, error) {
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
		rendered, err := renderWidget(ctx, childPath, child)
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
	if childHTML.Len() == 0 {
		return "", fmt.Errorf("%s.props.children: rendered content must not be empty for layout widget %q", path, widgetType)
	}

	childrenOut := template.HTML(childHTML.String())
	widgetID := strings.TrimSpace(widget.ID)
	if widgetID != "" && !isSafeHTMLID(widgetID) {
		return "", fmt.Errorf("%s.id: invalid id %q", path, widgetID)
	}

	switch widgetType {
	case "row":
		gapRaw, gapErr := readOptionalGap(widget.Props, path+".props.gap")
		if gapErr != nil {
			return "", gapErr
		}
		var rowStyle template.CSS
		if gapRaw != "" {
			rowStyle = template.CSS(fmt.Sprintf("gap:%s;", gapRaw))
		}
		return executeLayoutTemplate(ctx.WidgetTpl, "row", map[string]any{
			"ID":       widgetID,
			"Children": childrenOut,
			"Style":    rowStyle,
		})

	case "column":
		gapRaw, gapErr := readOptionalGap(widget.Props, path+".props.gap")
		if gapErr != nil {
			return "", gapErr
		}
		var colStyle template.CSS
		if gapRaw != "" {
			colStyle = template.CSS(fmt.Sprintf("gap:%s;", gapRaw))
		}
		return executeLayoutTemplate(ctx.WidgetTpl, "column", map[string]any{
			"ID":       widgetID,
			"Children": childrenOut,
			"Style":    colStyle,
		})

	case "grid":
		minCW, err := parseGridMinColumnWidth(widget.Props, path+".props.min_column_width")
		if err != nil {
			return "", err
		}
		gapRaw, gapErr := readOptionalGap(widget.Props, path+".props.gap")
		if gapErr != nil {
			return "", gapErr
		}
		gridGap := "1.25rem"
		if gapRaw != "" {
			gridGap = gapRaw
		}
		innerCSS := template.CSS(fmt.Sprintf("--widget-grid-min: %s; gap: %s;", minCW, gridGap))
		return executeLayoutTemplate(ctx.WidgetTpl, "grid", map[string]any{
			"ID":         widgetID,
			"Children":   childrenOut,
			"InnerStyle": innerCSS,
		})
	default:
		return "", fmt.Errorf("%s.type: unsupported layout widget %q", path, widgetType)
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

func renderLeafWidget(ctx *widgetRenderContext, path, widgetType string, widget WidgetNode) (string, error) {
	var buf bytes.Buffer
	switch widgetType {
	case "intro":
		data, err := parseIntroProps(ctx, widget, path)
		if err != nil {
			return "", err
		}
		if data == nil {
			return "", nil
		}
		if err := ctx.WidgetTpl.ExecuteTemplate(&buf, "intro", data); err != nil {
			return "", fmt.Errorf("cannot render intro: %w", err)
		}
	case "cover_banner":
		data, err := parseCoverBannerProps(ctx, widget, path)
		if err != nil {
			return "", err
		}
		if err := ctx.WidgetTpl.ExecuteTemplate(&buf, "cover_banner", data); err != nil {
			return "", fmt.Errorf("cannot render cover_banner: %w", err)
		}
	case "follow_us":
		data, err := buildFollowUsData(ctx, widget, path)
		if err != nil {
			return "", err
		}
		if data == nil {
			return "", nil
		}
		if err := ctx.WidgetTpl.ExecuteTemplate(&buf, "follow_us", data); err != nil {
			return "", fmt.Errorf("cannot render follow_us: %w", err)
		}
	case "info_grid":
		data, err := parseInfoGridProps(ctx, widget, path)
		if err != nil {
			return "", err
		}
		if err := ctx.WidgetTpl.ExecuteTemplate(&buf, "info_grid", data); err != nil {
			return "", fmt.Errorf("cannot render info_grid: %w", err)
		}
	case "images_grid":
		data, err := parseImagesGridProps(ctx, widget, path)
		if err != nil {
			return "", err
		}
		if err := ctx.WidgetTpl.ExecuteTemplate(&buf, "images_grid", data); err != nil {
			return "", fmt.Errorf("cannot render images_grid: %w", err)
		}
	case "careers_tabs":
		data, err := parseCareersTabsProps(ctx, widget, path)
		if err != nil {
			return "", err
		}
		if err := ctx.WidgetTpl.ExecuteTemplate(&buf, "careers_tabs", data); err != nil {
			return "", fmt.Errorf("cannot render careers_tabs: %w", err)
		}
	case "apps_showcase":
		id := strings.TrimSpace(widget.ID)
		if id != "" && !isSafeHTMLID(id) {
			return "", fmt.Errorf("%s.id: invalid id %q", path, id)
		}
		if err := ctx.WidgetTpl.ExecuteTemplate(&buf, "apps_showcase", struct{ ID string }{ID: id}); err != nil {
			return "", fmt.Errorf("cannot render apps_showcase: %w", err)
		}
	case "project_grid":
		id := strings.TrimSpace(widget.ID)
		if id != "" && !isSafeHTMLID(id) {
			return "", fmt.Errorf("%s.id: invalid id %q", path, id)
		}
		if err := ctx.WidgetTpl.ExecuteTemplate(&buf, "project_grid", struct{ ID string }{ID: id}); err != nil {
			return "", fmt.Errorf("cannot render project_grid: %w", err)
		}
	case "media_swiper":
		id := strings.TrimSpace(widget.ID)
		if id != "" && !isSafeHTMLID(id) {
			return "", fmt.Errorf("%s.id: invalid id %q", path, id)
		}
		if err := ctx.WidgetTpl.ExecuteTemplate(&buf, "media_swiper", struct{ ID string }{ID: id}); err != nil {
			return "", fmt.Errorf("cannot render media_swiper: %w", err)
		}
	default:
		return "", fmt.Errorf("%s.type: unsupported leaf widget %q", path, widgetType)
	}
	return buf.String(), nil
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

func parseImagesGridProps(ctx *widgetRenderContext, widget WidgetNode, path string) (*imagesGridTemplateData, error) {
	raw, ok := widget.Props["images"]
	if !ok {
		return nil, fmt.Errorf("%s.props.images: required and must not be empty", path)
	}
	entries, err := normalizeImagesGridRaw(raw, path+".props.images")
	if err != nil {
		return nil, err
	}
	if len(entries) == 0 {
		return nil, fmt.Errorf("%s.props.images: required and must not be empty", path)
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
	return &imagesGridTemplateData{Title: strings.TrimSpace(top.Title), Images: entries}, nil
}

func normalizeImagesGridRaw(raw json.RawMessage, cfgPath string) ([]imagesGridEntry, error) {
	var asStrings []string
	if err := json.Unmarshal(raw, &asStrings); err == nil {
		var out []imagesGridEntry
		for i, s := range asStrings {
			s = strings.TrimSpace(s)
			if s == "" {
				return nil, fmt.Errorf("%s[%d]: empty string not allowed", cfgPath, i)
			}
			out = append(out, imagesGridEntry{Src: s, Alt: fmt.Sprintf("photo %d", i+1)})
		}
		return out, nil
	}
	var asObjs []struct {
		Src string `json:"src"`
		Alt string `json:"alt"`
	}
	if err := json.Unmarshal(raw, &asObjs); err != nil {
		return nil, fmt.Errorf("%s: must be an array of strings or objects with src", cfgPath)
	}
	var out []imagesGridEntry
	for i, o := range asObjs {
		src := strings.TrimSpace(o.Src)
		if src == "" {
			return nil, fmt.Errorf("%s[%d].src: required", cfgPath, i)
		}
		alt := strings.TrimSpace(o.Alt)
		if alt == "" {
			alt = fmt.Sprintf("photo %d", i+1)
		}
		out = append(out, imagesGridEntry{Src: src, Alt: alt})
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
