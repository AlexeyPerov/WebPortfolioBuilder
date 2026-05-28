package main

import (
	"encoding/json"
	"fmt"
	"html"
	"strings"
)

const defaultGoogleFontsHref = "https://fonts.googleapis.com/css?family=Quicksand:700|Roboto:400,400i,700&display=swap"

func normalizedTypography(t TypographyConfig) (href, headingStack, bodyStack string) {
	href = strings.TrimSpace(t.GoogleFontsStylesheetHref)
	if href == "" {
		href = defaultGoogleFontsHref
	}
	headingStack = strings.TrimSpace(t.FontFamilyHeading)
	if headingStack == "" {
		headingStack = `"Quicksand", sans-serif`
	}
	bodyStack = strings.TrimSpace(t.FontFamilyBody)
	if bodyStack == "" {
		bodyStack = `"Roboto", sans-serif`
	}
	return href, headingStack, bodyStack
}

func buildFooterOuterHTML(f FooterConfig) string {
	if !f.isFooterEnabled() {
		return ""
	}
	inner := buildFooterSection(f)
	return `<footer class="footer section section-gradient" id="footer"><div class="container">` + inner + `</div></footer>`
}

type widgetsExportJSON struct {
	ScrollReveal struct {
		RespectReducedMotion bool    `json:"respect_reduced_motion"`
		RootMargin           string  `json:"root_margin"`
		Threshold            float64 `json:"threshold"`
	} `json:"scroll_reveal"`
	Carousel struct {
		SwipeThresholdPx   int  `json:"swipe_threshold_px"`
		KeyboardNavigation bool `json:"keyboard_navigation"`
	} `json:"carousel"`
	SplitWidget struct {
		KeyboardNavigation bool `json:"keyboard_navigation"`
	} `json:"split_widget"`
}

func buildWidgetsConfigScript(w WidgetsConfig) string {
	var j widgetsExportJSON
	j.ScrollReveal.RespectReducedMotion = true
	if w.ScrollReveal.RespectReducedMotion != nil {
		j.ScrollReveal.RespectReducedMotion = *w.ScrollReveal.RespectReducedMotion
	}
	j.ScrollReveal.RootMargin = "0px 0px -5% 0px"
	if rm := strings.TrimSpace(w.ScrollReveal.RootMargin); rm != "" {
		j.ScrollReveal.RootMargin = rm
	}
	j.ScrollReveal.Threshold = 0.12
	if w.ScrollReveal.Threshold != nil {
		j.ScrollReveal.Threshold = *w.ScrollReveal.Threshold
	}
	j.Carousel.SwipeThresholdPx = 30
	if w.Carousel.SwipeThresholdPx != nil && *w.Carousel.SwipeThresholdPx > 0 {
		j.Carousel.SwipeThresholdPx = *w.Carousel.SwipeThresholdPx
	}
	j.Carousel.KeyboardNavigation = true
	if w.Carousel.KeyboardNavigation != nil {
		j.Carousel.KeyboardNavigation = *w.Carousel.KeyboardNavigation
	}
	j.SplitWidget.KeyboardNavigation = false
	if w.SplitWidget.KeyboardNavigation != nil {
		j.SplitWidget.KeyboardNavigation = *w.SplitWidget.KeyboardNavigation
	}
	b, err := json.Marshal(j)
	if err != nil {
		b = []byte("{}")
	}
	safe := strings.ReplaceAll(string(b), "<", "\\u003c")
	return `<script type="application/json" id="site-widgets-config">` + safe + `</script>`
}

func catalogStatLineOr(value, fallback string) string {
	if strings.TrimSpace(value) != "" {
		return value
	}
	return fallback
}

func catalogStoreBtnIconImg(src string) string {
	return fmt.Sprintf(
		`<img class="catalog-store-btn__icon" src="%s" alt="" decoding="async">`,
		html.EscapeString(src))
}

type resolvedStoreEntry struct {
	URL, AriaLabel, IconSrc, ClassSuffix string
}

var presetStoreIconKeyToClass = map[string]string{
	"google_play": "googleplay",
	"app_store":   "appstore",
	"amazon":      "amazon",
	"galaxy":      "galaxy",
}

func normalizeStoreIconKey(icon string) string {
	return strings.TrimSpace(strings.ToLower(strings.ReplaceAll(icon, " ", "_")))
}

func sanitizeIconClassKey(s string) string {
	s = strings.TrimSpace(strings.ToLower(s))
	var b strings.Builder
	for _, r := range s {
		switch {
		case r >= 'a' && r <= 'z', r >= '0' && r <= '9':
			b.WriteRune(r)
		case r == '_' || r == '-':
			b.WriteByte('-')
		}
	}
	out := strings.Trim(b.String(), "-")
	if out == "" {
		return "custom"
	}
	return out
}

func storeClassSuffixFromLink(iconKey string, hasCustomImage bool) string {
	if iconKey != "" {
		if suffix, ok := presetStoreIconKeyToClass[iconKey]; ok {
			return suffix
		}
		return sanitizeIconClassKey(iconKey)
	}
	if hasCustomImage {
		return "custom"
	}
	return "custom"
}

func defaultAriaForStoreIconKey(iconKey string) string {
	switch iconKey {
	case "google_play":
		return "Get it on Google Play"
	case "app_store":
		return "Download on the App Store"
	case "amazon":
		return "Available at Amazon Appstore"
	case "galaxy":
		return "Available on Galaxy Store"
	default:
		return ""
	}
}

type legacyStoreURLField struct {
	jsonKey   string
	presetKey string
}

var legacyStoreURLFields = []legacyStoreURLField{
	{jsonKey: "google_play_url", presetKey: "google_play"},
	{jsonKey: "app_store_url", presetKey: "app_store"},
	{jsonKey: "amazon_store_url", presetKey: "amazon"},
	{jsonKey: "galaxy_store_url", presetKey: "galaxy"},
}

// catalogStoreURLWarnings reports presets referenced in JSON with an empty URL after trim.
func catalogStoreURLWarnings(pagePath string, appPath string, appRaw json.RawMessage) []ConfigWarning {
	var obj map[string]json.RawMessage
	if err := json.Unmarshal(appRaw, &obj); err != nil {
		return nil
	}
	var warnings []ConfigWarning
	for _, field := range legacyStoreURLFields {
		raw, ok := obj[field.jsonKey]
		if !ok {
			continue
		}
		var url string
		if err := json.Unmarshal(raw, &url); err != nil {
			continue
		}
		if strings.TrimSpace(url) != "" {
			continue
		}
		warnings = append(warnings, contentWarning(pagePath,
			fmt.Sprintf("%s.%s: store preset %q referenced but URL is empty", appPath, field.jsonKey, field.presetKey)))
	}
	rawLinks, ok := obj["store_links"]
	if !ok {
		return warnings
	}
	var links []StoreLink
	if err := json.Unmarshal(rawLinks, &links); err != nil {
		return warnings
	}
	for i, link := range links {
		url := strings.TrimSpace(link.URL)
		iconKey := normalizeStoreIconKey(link.Icon)
		iconImage := strings.TrimSpace(link.IconImage)
		if url != "" {
			continue
		}
		if iconKey == "" && iconImage == "" {
			continue
		}
		preset := iconKey
		if preset == "" {
			preset = "custom"
		}
		warnings = append(warnings, contentWarning(pagePath,
			fmt.Sprintf("%s.store_links[%d]: store preset %q referenced but URL is empty", appPath, i, preset)))
	}
	return warnings
}

func resolveCatalogStoreEntries(app CatalogApp, icons StoreIcons) []resolvedStoreEntry {
	if len(app.StoreLinks) > 0 {
		var out []resolvedStoreEntry
		for _, link := range app.StoreLinks {
			u := strings.TrimSpace(link.URL)
			if u == "" {
				continue
			}
			imgPath := strings.TrimSpace(link.IconImage)
			iconKey := normalizeStoreIconKey(link.Icon)
			var iconSrc string
			if imgPath != "" {
				iconSrc = imgPath
			} else if iconKey != "" {
				if p, ok := icons[iconKey]; ok {
					iconSrc = strings.TrimSpace(p)
				}
			}
			if iconSrc == "" {
				continue
			}
			aria := strings.TrimSpace(link.AriaLabel)
			if aria == "" {
				aria = defaultAriaForStoreIconKey(iconKey)
				if aria == "" {
					aria = "Store link"
				}
			}
			classSuffix := storeClassSuffixFromLink(iconKey, imgPath != "")
			out = append(out, resolvedStoreEntry{
				URL: u, AriaLabel: aria, IconSrc: iconSrc, ClassSuffix: classSuffix,
			})
		}
		return out
	}
	type legacySlot struct {
		url, iconKey, classSuffix, defAria string
	}
	slots := []legacySlot{
		{app.GooglePlayURL, "google_play", "googleplay", "Get it on Google Play"},
		{app.AppStoreURL, "app_store", "appstore", "Download on the App Store"},
		{app.AmazonStoreURL, "amazon", "amazon", "Available at Amazon Appstore"},
		{app.GalaxyStoreURL, "galaxy", "galaxy", "Available on Galaxy Store"},
	}
	var out []resolvedStoreEntry
	for _, slot := range slots {
		u := strings.TrimSpace(slot.url)
		if u == "" {
			continue
		}
		iconSrc := strings.TrimSpace(icons[slot.iconKey])
		if iconSrc == "" {
			continue
		}
		out = append(out, resolvedStoreEntry{
			URL: u, AriaLabel: slot.defAria, IconSrc: iconSrc, ClassSuffix: slot.classSuffix,
		})
	}
	return out
}

func buildCatalogStoreRow(app CatalogApp, icons StoreIcons) string {
	entries := resolveCatalogStoreEntries(app, icons)
	if len(entries) == 0 {
		return ""
	}
	var b strings.Builder
	for _, e := range entries {
		if strings.TrimSpace(e.URL) == "" {
			continue
		}
		b.WriteString(fmt.Sprintf(
			`<a class="catalog-store-btn catalog-store-btn--%s" href="%s"%s aria-label="%s">%s</a>`,
			html.EscapeString(e.ClassSuffix),
			html.EscapeString(e.URL),
			externalLinkAttrs(e.URL),
			html.EscapeString(e.AriaLabel),
			catalogStoreBtnIconImg(e.IconSrc)))
	}
	return `<div class="catalog-app-card__stores">` + b.String() + `</div>`
}

func buildFooterContactRow(c FooterContact) string {
	var textParts []string
	for _, p := range c.Paragraphs {
		p = strings.TrimSpace(p)
		if p != "" {
			textParts = append(textParts, html.EscapeString(p))
		}
	}
	textJoined := strings.Join(textParts, " ")
	email := strings.TrimSpace(c.Email)
	if textJoined == "" && email == "" {
		return ""
	}

	var b strings.Builder
	b.WriteString(`<div class="footer-contact-row" id="footer-contact">`)
	b.WriteString(`<div class="footer-contact-row__text">`)
	b.WriteString(textJoined)
	b.WriteString(`</div>`)
	if email != "" {
		b.WriteString(fmt.Sprintf(
			`<a class="footer-contact-row__email" href="mailto:%s">%s</a>`,
			html.EscapeString(email), html.EscapeString(email)))
	}
	b.WriteString(`</div>`)
	return b.String()
}

func externalLinkAttrs(href string) string {
	if strings.HasPrefix(href, "http://") || strings.HasPrefix(href, "https://") {
		return ` target="_blank" rel="noopener noreferrer"`
	}
	return ""
}

func buildFooterLegalRow(f FooterConfig) string {
	privacyURL := strings.TrimSpace(f.PrivacyURL)
	termsURL := strings.TrimSpace(f.TermsURL)
	if privacyURL == "" && termsURL == "" {
		return ""
	}

	privacyLabel := strings.TrimSpace(f.PrivacyLabel)
	if privacyLabel == "" {
		privacyLabel = "Privacy Policy"
	}
	termsLabel := strings.TrimSpace(f.TermsLabel)
	if termsLabel == "" {
		termsLabel = "Terms of Service"
	}

	var b strings.Builder
	b.WriteString(`<div class="footer-legal-links">`)
	if privacyURL != "" {
		b.WriteString(fmt.Sprintf(
			`<a class="footer-legal-links__left" href="%s"%s>%s</a>`,
			html.EscapeString(privacyURL), externalLinkAttrs(privacyURL), html.EscapeString(privacyLabel)))
	}
	if termsURL != "" {
		b.WriteString(fmt.Sprintf(
			`<a class="footer-legal-links__right" href="%s"%s>%s</a>`,
			html.EscapeString(termsURL), externalLinkAttrs(termsURL), html.EscapeString(termsLabel)))
	}
	b.WriteString(`</div>`)
	return b.String()
}

func buildFooterSection(f FooterConfig) string {
	contactRow := buildFooterContactRow(f.Contact)
	legalRow := buildFooterLegalRow(f)
	if strings.TrimSpace(f.SectionTitle) == "" && contactRow == "" && legalRow == "" && strings.TrimSpace(f.Copyright) == "" && strings.TrimSpace(f.CookieNotice) == "" {
		return ""
	}

	var b strings.Builder
	if t := strings.TrimSpace(f.SectionTitle); t != "" {
		b.WriteString(fmt.Sprintf(`<h2 class="center footer-heading">%s</h2>`, html.EscapeString(t)))
	}
	if contactRow != "" {
		b.WriteString(contactRow)
	}
	if legalRow != "" {
		b.WriteString(legalRow)
	}
	if t := strings.TrimSpace(f.Copyright); t != "" {
		b.WriteString(fmt.Sprintf(`<p class="footer-copyright">%s</p>`, html.EscapeString(t)))
	}
	if t := strings.TrimSpace(f.CookieNotice); t != "" {
		b.WriteString(fmt.Sprintf(`<p class="footer-cookie">%s</p>`, html.EscapeString(t)))
	}
	return b.String()
}

func buildList(items []string) string {
	if len(items) == 0 {
		return ""
	}
	parts := make([]string, 0, len(items))
	for _, item := range items {
		parts = append(parts, "<li>"+html.EscapeString(item)+"</li>")
	}
	return strings.Join(parts, "")
}

const (
	svgGitHubIcon = `<svg class="follow-us__icon" viewBox="0 0 24 24" width="28" height="28" aria-hidden="true"><path fill="currentColor" d="M12 0C5.374 0 0 5.373 0 12c0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23A11.509 11.509 0 0112 5.803c1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576C20.566 21.797 24 17.3 24 12c0-6.627-5.373-12-12-12z"/></svg>`

	svgLinkedInIcon = `<svg class="follow-us__icon" viewBox="0 0 24 24" width="28" height="28" aria-hidden="true"><path fill="currentColor" d="M20.447 20.452h-3.554v-5.569c0-1.328-.027-3.037-1.852-3.037-1.853 0-2.136 1.445-2.136 2.939v5.667H9.351V9h3.414v1.561h.046c.477-.9 1.637-1.85 3.37-1.85 3.601 0 4.267 2.37 4.267 5.455v6.286zM5.337 7.433c-1.144 0-2.063-.926-2.063-2.065 0-1.138.92-2.063 2.063-2.063 1.14 0 2.064.925 2.064 2.063 0 1.139-.925 2.065-2.064 2.065zm1.782 13.019H3.555V9h3.564v11.452zM22.225 0H1.771C.792 0 0 .774 0 1.729v20.542C0 23.227.792 24 1.771 24h20.451C23.2 24 24 23.227 24 22.271V1.729C24 .774 23.2 0 22.222 0h.003z"/></svg>`

	svgFacebookIcon = `<svg class="follow-us__icon" viewBox="0 0 24 24" width="28" height="28" aria-hidden="true"><path fill="currentColor" d="M24 12.073C24 5.446 18.627.073 12 .073S0 5.446 0 12.073c0 5.99 4.388 10.954 10.125 11.854v-8.385H7.078v-3.47h3.047V9.43c0-3.007 1.792-4.669 4.533-4.669 1.312 0 2.686.235 2.686.235v2.953H15.83c-1.491 0-1.956.925-1.956 1.874v2.25h3.328l-.532 3.47h-2.796v8.385C19.612 23.027 24 18.062 24 12.073z"/></svg>`
)

func socialIconPresetClass(icon string) string {
	switch strings.ToLower(strings.TrimSpace(icon)) {
	case "github":
		return "follow-us__btn--github"
	case "linkedin":
		return "follow-us__btn--linkedin"
	case "facebook":
		return "follow-us__btn--facebook"
	default:
		return ""
	}
}

func socialIconPresetSVG(icon string) string {
	switch strings.ToLower(strings.TrimSpace(icon)) {
	case "github":
		return svgGitHubIcon
	case "linkedin":
		return svgLinkedInIcon
	case "facebook":
		return svgFacebookIcon
	default:
		return ""
	}
}

func ariaDefaultForSocialPreset(icon string) string {
	switch strings.ToLower(strings.TrimSpace(icon)) {
	case "github":
		return "GitHub"
	case "linkedin":
		return "LinkedIn"
	case "facebook":
		return "Facebook"
	default:
		return "Link"
	}
}
