package main

import (
	"bytes"
	"fmt"
	"html/template"
	"os"
	"path/filepath"
	"sort"
	"strings"
)

type renderedNavItem struct {
	Label      string
	Href       string
	OpenNewTab bool
}

type renderedPageData struct {
	Title                  string
	MetaDescription        string
	CanonicalURL           string
	OpenGraphImage         string
	HasSEO                 bool
	TypographyGoogleFonts  string
	TypographyFontHeading  string
	TypographyFontBody     string
	ThemeCSSVariables      string
	SiteIconHref           string
	ShowHeader             bool
	ShowFooter             bool
	HeaderBrandHref        string
	HeaderBrandLogo        string
	HeaderBrandText        string
	HeaderNav              []renderedNavItem
	MainContentHTML        template.HTML
	FooterHTML             template.HTML
	StylesHref             string
	ScrollRevealScriptHref string
	GameSwiperScriptHref   string
	SplitWidgetScriptHref  string
}

func renderSiteBundle(bundle SiteBundle, targetDir, templateDir string) error {
	routes, err := buildRouteIndex(bundle)
	if err != nil {
		return err
	}

	layoutPath := filepath.Join(templateDir, "layout.html")
	layoutTpl, err := template.ParseFiles(layoutPath)
	if err != nil {
		return fmt.Errorf("cannot parse layout template %q: %w", layoutPath, err)
	}

	pageByPath := make(map[string]SitePageFile, len(bundle.Pages))
	for _, page := range bundle.Pages {
		pageByPath[page.Path] = page
	}

	for _, route := range routes.Ordered {
		pageFile := pageByPath[route.SourcePath]
		data, err := buildRenderedPageData(bundle, pageFile, route, routes)
		if err != nil {
			return err
		}

		var out bytes.Buffer
		if err := layoutTpl.Execute(&out, data); err != nil {
			return fmt.Errorf("cannot render page %q: %w", route.SourcePath, err)
		}

		dst := filepath.Join(targetDir, route.OutputRelPath)
		if err := os.MkdirAll(filepath.Dir(dst), 0o755); err != nil {
			return fmt.Errorf("cannot create directory for %q: %w", dst, err)
		}
		if err := os.WriteFile(dst, out.Bytes(), 0o644); err != nil {
			return fmt.Errorf("cannot write page %q: %w", dst, err)
		}
	}

	return nil
}

func buildRenderedPageData(bundle SiteBundle, pageFile SitePageFile, route PageRoute, routes RouteIndex) (renderedPageData, error) {
	page := pageFile.Page
	data := renderedPageData{}

	title := strings.TrimSpace(page.Title)
	if title == "" {
		title = strings.TrimSpace(bundle.Site.SiteID)
	}
	data.Title = title

	data.MetaDescription = strings.TrimSpace(page.SEO.Description)
	data.CanonicalURL = resolvedCanonicalURL(bundle.Site.BaseURL, route, strings.TrimSpace(page.SEO.CanonicalURL))
	data.OpenGraphImage = resolvedOpenGraphImage(bundle.Site.BaseURL, strings.TrimSpace(page.SEO.OGImage))
	data.HasSEO = data.MetaDescription != "" || data.CanonicalURL != "" || data.OpenGraphImage != ""

	fontsHref, fontHeading, fontBody := normalizedTypography(bundle.Site.Typography)
	data.TypographyGoogleFonts = fontsHref
	data.TypographyFontHeading = fontHeading
	data.TypographyFontBody = fontBody
	data.ThemeCSSVariables = buildThemeCSSVariables(bundle.Site.Theme)

	data.ShowHeader = !page.Layout.HideHeader
	data.ShowFooter = bundle.Site.Footer.isFooterEnabled() && !page.Layout.HideFooter

	brandHref, err := resolveInternalSlugReference(route, "", routes.BySlug)
	if err != nil {
		return renderedPageData{}, fmt.Errorf("%s -> header.brand: %w", bundle.SitePath, err)
	}
	data.HeaderBrandHref = brandHref
	data.HeaderBrandLogo = resolveAssetHrefForPage(bundle.Site.Header.Brand.Logo, route)
	data.HeaderBrandText = strings.TrimSpace(bundle.Site.Header.Brand.Text)
	data.SiteIconHref = data.HeaderBrandLogo

	navItems, err := renderHeaderNavForPage(bundle, route, routes)
	if err != nil {
		return renderedPageData{}, err
	}
	data.HeaderNav = navItems

	mainContent, err := renderWidgetTree(pageFile.Path, page.Widgets)
	if err != nil {
		return renderedPageData{}, err
	}
	data.MainContentHTML = mainContent
	if data.ShowFooter {
		data.FooterHTML = template.HTML(buildFooterOuterHTML(bundle.Site.Footer))
	}

	assetPrefix := assetPrefixForDepth(route.DirRelPath)
	data.StylesHref = assetPrefix + "styles.css"
	data.ScrollRevealScriptHref = assetPrefix + "scroll-reveal.js"
	data.GameSwiperScriptHref = assetPrefix + "game-swiper.js"
	data.SplitWidgetScriptHref = assetPrefix + "split-widget.js"

	return data, nil
}

func renderHeaderNavForPage(bundle SiteBundle, route PageRoute, routes RouteIndex) ([]renderedNavItem, error) {
	var out []renderedNavItem
	for i, item := range bundle.Site.Header.Nav {
		label := strings.TrimSpace(item.Label)
		href := strings.TrimSpace(item.Href)
		if label == "" || href == "" {
			continue
		}

		resolved, err := resolveInternalSlugReference(route, href, routes.BySlug)
		if err != nil {
			return nil, fmt.Errorf(`%s -> header.nav[%d].href: %w`, bundle.SitePath, i, err)
		}

		out = append(out, renderedNavItem{
			Label:      label,
			Href:       resolved,
			OpenNewTab: item.OpenInNewTab && strings.HasPrefix(strings.ToLower(resolved), "http"),
		})
	}
	return out, nil
}

func buildThemeCSSVariables(theme map[string]string) string {
	keys := make([]string, 0, len(theme))
	for key := range theme {
		keys = append(keys, key)
	}
	sort.Strings(keys)

	var b strings.Builder
	for _, key := range keys {
		val := strings.TrimSpace(theme[key])
		if val == "" {
			continue
		}
		if b.Len() > 0 {
			b.WriteString("\n")
		}
		b.WriteString("      --")
		b.WriteString(key)
		b.WriteString(": ")
		b.WriteString(val)
		b.WriteString(";")
	}
	return b.String()
}

func resolvedCanonicalURL(baseURL string, route PageRoute, explicit string) string {
	explicit = strings.TrimSpace(explicit)
	if explicit != "" {
		return explicit
	}
	base := strings.TrimSpace(baseURL)
	if base == "" {
		return ""
	}
	base = strings.TrimRight(base, "/")
	if route.Slug == "" {
		return base + "/"
	}
	return base + "/" + route.Slug + "/"
}

func resolvedOpenGraphImage(baseURL, rawImage string) string {
	rawImage = strings.TrimSpace(rawImage)
	if rawImage == "" {
		return ""
	}
	if isExternalOrSpecialHref(rawImage) || strings.HasPrefix(rawImage, "/") {
		return rawImage
	}
	base := strings.TrimRight(strings.TrimSpace(baseURL), "/")
	if base == "" {
		return rawImage
	}
	return base + "/" + strings.TrimPrefix(rawImage, "./")
}
