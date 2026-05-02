package main

import (
	"fmt"
	"html"
	"strings"
)

func buildPlaceholders(projectRoot string, config Config) map[string]string {
	values := map[string]string{}

	for key, value := range config.Content {
		values[key] = value
	}
	for key, value := range config.Theme {
		values["theme_"+key] = value
	}

	values["game_store_icon_height"] = gameStoreIconHeightCSS(projectRoot, config.GameStoreIcons.withDefaults())

	values["games_columns"] = buildGamesColumns(projectRoot, config.Games, config.GameStoreIcons.withDefaults(), config.GameSubscribe)
	values["offers_items"] = buildOffersItems(config.Offers)
	values["vacancies_section"] = buildVacanciesSection(config.Vacancies, config.Content)
	values["photos_items"] = buildPhotosItems(config.Photos)
	values["footer_section"] = buildFooterSection(config.Footer)
	values["cover_banner_section"] = buildCoverBannerSection(config.Content)
	values["follow_us_section"] = buildFollowUsSection(config.Content, config.Social)
	values["header_nav"] = buildHeaderNav(config.Content, len(config.Vacancies) > 0)
	values["header_brand_row"] = buildHeaderBrandRowHTML(config.Content)

	return values
}

// buildHeaderBrandRowHTML renders the header logo and/or title. When header_logo_image is set,
// only the image is shown; header_brand_name is omitted entirely.
func buildHeaderBrandRowHTML(c map[string]string) string {
	logo := strings.TrimSpace(c["header_logo_image"])
	brand := strings.TrimSpace(c["header_brand_name"])
	if logo != "" {
		return fmt.Sprintf(`<a class="brand-home" href="#" aria-label="Back to top"><img class="logo" src="%s" alt="Site logo"></a>`,
			html.EscapeString(logo))
	}
	if brand != "" {
		return fmt.Sprintf(`<span class="brand-name">%s</span>`, html.EscapeString(brand))
	}
	return ""
}

func buildHeaderNav(c map[string]string, includeVacancies bool) string {
	type navItem struct {
		href, labelKey string
	}
	items := []navItem{
		{"#intro_title", "nav_about"},
		{"#games_title", "nav_games"},
		{"#offers_title", "nav_we_offer"},
	}
	if includeVacancies {
		items = append(items, navItem{"#vacancies_title", "nav_vacancies"})
	}
	items = append(items, navItem{"#follow_us_title", "nav_contact"})
	var parts []string
	for _, it := range items {
		label := strings.TrimSpace(c[it.labelKey])
		if label == "" {
			continue
		}
		parts = append(parts, fmt.Sprintf(
			`<a class="site-nav__link" href="%s">%s</a>`,
			it.href,
			html.EscapeString(label)))
	}
	if len(parts) == 0 {
		return ""
	}
	return `<nav class="site-nav" aria-label="On this page">` + strings.Join(parts, "") + `</nav>`
}

func gameStatLineOr(value, fallback string) string {
	if strings.TrimSpace(value) != "" {
		return value
	}
	return fallback
}

func buildGameSubscribeWidget(s GameSubscribeBlock) string {
	var items []string
	for _, link := range s.Links {
		u := strings.TrimSpace(link.URL)
		if u == "" {
			continue
		}
		label := strings.TrimSpace(link.Label)
		if label == "" {
			label = u
		}
		items = append(items, fmt.Sprintf(
			`<li class="game-card-full__subscribe-item"><a class="game-card-full__subscribe-link" href="%s"%s>%s</a></li>`,
			html.EscapeString(u),
			externalLinkAttrs(u),
			html.EscapeString(label)))
	}
	if len(items) == 0 {
		return ""
	}
	title := strings.TrimSpace(s.Title)
	if title == "" {
		title = "Subscribe for news"
	}
	return fmt.Sprintf(
		`<div class="game-card-full__subscribe"><p class="game-card-full__subscribe-title">%s</p><ul class="game-card-full__subscribe-list">%s</ul></div>`,
		html.EscapeString(title),
		strings.Join(items, ""))
}

func buildGamesColumns(projectRoot string, games []Game, icons GameStoreIcons, subscribe GameSubscribeBlock) string {
	if len(games) == 0 {
		return ""
	}

	subscribeHTML := buildGameSubscribeWidget(subscribe)
	parts := make([]string, 0, len(games))
	for _, game := range games {
		stores := buildGameStoreRow(game, icons)
		imgAttrs := gameIconImgSizeAttrs(projectRoot, game.Image)
		headerHTML, bodyTitleHTML, titleInHeader := buildGameHeaderAndTitleHTML(game)
		swiperHTML := buildGameSwiperHTML(game)
		cardBGAttr := gameCardBackgroundStyleAttr(game)
		cardClassExtra := ""
		if titleInHeader {
			cardClassExtra = " game-card-full--title-in-header"
		}
		left1 := gameStatLineOr(game.StatLeftLine1, "1M+")
		left2 := gameStatLineOr(game.StatLeftLine2, "Downloads")
		right1 := gameStatLineOr(game.StatRightLine1, "4.8")
		right2 := gameStatLineOr(game.StatRightLine2, "on Google Play")
		part := fmt.Sprintf(
			`<article class="offer-card game-card-full scroll-reveal%s"%s>%s<div class="game-card-full__icon-row"><div class="game-card-full__stat game-card-full__stat--left"><span class="game-card-full__stat-line game-card-full__stat-line--emphasis">%s</span><span class="game-card-full__stat-line">%s</span></div><div class="game-card-full__icon-wrap"><img class="game-icon"%s src="%s" alt="%s"></div><div class="game-card-full__stat game-card-full__stat--right"><span class="game-card-full__stat-line game-card-full__stat-line--emphasis">%s</span><span class="game-card-full__stat-line">%s</span></div></div><div class="game-card-full__body">%s<div class="game-card-full__columns"><p class="game-card-full__col">%s</p><p class="game-card-full__col">%s</p></div></div>%s%s%s</article>`,
			cardClassExtra,
			cardBGAttr,
			headerHTML,
			html.EscapeString(left1),
			html.EscapeString(left2),
			imgAttrs,
			html.EscapeString(game.Image),
			html.EscapeString(game.Title),
			html.EscapeString(right1),
			html.EscapeString(right2),
			bodyTitleHTML,
			html.EscapeString(game.Text1),
			html.EscapeString(game.Text2),
			swiperHTML,
			stores,
			subscribeHTML,
		)
		parts = append(parts, part)
	}
	return strings.Join(parts, "\n")
}

func gameCardBackgroundStyleAttr(g Game) string {
	bg := strings.TrimSpace(g.CardBackground)
	if bg == "" {
		bg = "var(--widget-gradient)"
	}
	return fmt.Sprintf(` style="background: %s"`, html.EscapeString(bg))
}

func buildGameHeaderImageHTML(g Game) string {
	path := strings.TrimSpace(g.HeaderImage)
	if path == "" {
		return ""
	}
	alt := strings.TrimSpace(g.Title)
	if alt == "" {
		alt = "Game"
	}
	return fmt.Sprintf(
		`<div class="game-card-full__header"><img class="game-card-full__header-image" src="%s" alt="%s" decoding="async"></div>`,
		html.EscapeString(path),
		html.EscapeString(alt))
}

func buildGameHeaderAndTitleHTML(g Game) (headerHTML, bodyTitleHTML string, titleInHeader bool) {
	title := strings.TrimSpace(g.Title)
	if strings.TrimSpace(g.HeaderImage) != "" {
		if title != "" {
			return buildGameHeaderImageHTML(g), fmt.Sprintf(`<h3>%s</h3>`, html.EscapeString(title)), false
		}
		return buildGameHeaderImageHTML(g), "", false
	}
	if title == "" {
		return "", "", false
	}
	return fmt.Sprintf(`<div class="game-card-full__header"><h3 class="game-card-full__header-title">%s</h3></div>`, html.EscapeString(title)), "", true
}

func buildGameSwiperHTML(g Game) string {
	images := make([]string, 0, len(g.SwiperImages))
	for _, p := range g.SwiperImages {
		p = strings.TrimSpace(p)
		if p != "" {
			images = append(images, p)
		}
	}
	if len(images) == 0 {
		return ""
	}
	var slides strings.Builder
	for i, src := range images {
		slides.WriteString(fmt.Sprintf(
			`<div class="game-swiper__slide"><img class="game-swiper__img" src="%s" alt="%s screenshot %d" loading="lazy" decoding="async"></div>`,
			html.EscapeString(src),
			html.EscapeString(strings.TrimSpace(g.Title)),
			i+1,
		))
	}
	return fmt.Sprintf(
		`<div class="game-swiper" data-game-swiper><button class="game-swiper__arrow game-swiper__arrow--prev" type="button" aria-label="Previous screenshot">‹</button><div class="game-swiper__viewport"><div class="game-swiper__track">%s</div></div><button class="game-swiper__arrow game-swiper__arrow--next" type="button" aria-label="Next screenshot">›</button></div>`,
		slides.String(),
	)
}

func gameStoreIconImg(src string) string {
	return fmt.Sprintf(
		`<img class="game-store-btn__icon" src="%s" alt="" decoding="async">`,
		html.EscapeString(src))
}

func buildGameStoreRow(g Game, icons GameStoreIcons) string {
	var b strings.Builder
	if u := strings.TrimSpace(g.GooglePlayURL); u != "" {
		b.WriteString(fmt.Sprintf(
			`<a class="game-store-btn game-store-btn--googleplay" href="%s"%s aria-label="Get it on Google Play">%s</a>`,
			html.EscapeString(u), externalLinkAttrs(u), gameStoreIconImg(icons.GooglePlay)))
	}
	if u := strings.TrimSpace(g.AppStoreURL); u != "" {
		b.WriteString(fmt.Sprintf(
			`<a class="game-store-btn game-store-btn--appstore" href="%s"%s aria-label="Download on the App Store">%s</a>`,
			html.EscapeString(u), externalLinkAttrs(u), gameStoreIconImg(icons.AppStore)))
	}
	if u := strings.TrimSpace(g.AmazonStoreURL); u != "" {
		b.WriteString(fmt.Sprintf(
			`<a class="game-store-btn game-store-btn--amazon" href="%s"%s aria-label="Available at Amazon Appstore">%s</a>`,
			html.EscapeString(u), externalLinkAttrs(u), gameStoreIconImg(icons.Amazon)))
	}
	if u := strings.TrimSpace(g.GalaxyStoreURL); u != "" {
		b.WriteString(fmt.Sprintf(
			`<a class="game-store-btn game-store-btn--galaxy" href="%s"%s aria-label="Available on Galaxy Store">%s</a>`,
			html.EscapeString(u), externalLinkAttrs(u), gameStoreIconImg(icons.Galaxy)))
	}
	if b.Len() == 0 {
		return ""
	}
	return `<div class="game-card-full__stores">` + b.String() + `</div>`
}

func buildCoverBannerSection(content map[string]string) string {
	path := strings.TrimSpace(content["cover_image"])
	if path == "" {
		return ""
	}
	alt := strings.TrimSpace(content["cover_image_alt"])
	if alt == "" {
		alt = "Cover"
	}
	return fmt.Sprintf(
		`<section class="cover-banner scroll-reveal" aria-label="Cover"><img class="cover-banner__img" src="%s" alt="%s"></section>`,
		html.EscapeString(path),
		html.EscapeString(alt),
	)
}

const (
	svgGitHubIcon = `<svg class="follow-us__icon" viewBox="0 0 24 24" width="28" height="28" aria-hidden="true"><path fill="currentColor" d="M12 0C5.374 0 0 5.373 0 12c0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23A11.509 11.509 0 0112 5.803c1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576C20.566 21.797 24 17.3 24 12c0-6.627-5.373-12-12-12z"/></svg>`

	svgLinkedInIcon = `<svg class="follow-us__icon" viewBox="0 0 24 24" width="28" height="28" aria-hidden="true"><path fill="currentColor" d="M20.447 20.452h-3.554v-5.569c0-1.328-.027-3.037-1.852-3.037-1.853 0-2.136 1.445-2.136 2.939v5.667H9.351V9h3.414v1.561h.046c.477-.9 1.637-1.85 3.37-1.85 3.601 0 4.267 2.37 4.267 5.455v6.286zM5.337 7.433c-1.144 0-2.063-.926-2.063-2.065 0-1.138.92-2.063 2.063-2.063 1.14 0 2.064.925 2.064 2.063 0 1.139-.925 2.065-2.064 2.065zm1.782 13.019H3.555V9h3.564v11.452zM22.225 0H1.771C.792 0 0 .774 0 1.729v20.542C0 23.227.792 24 1.771 24h20.451C23.2 24 24 23.227 24 22.271V1.729C24 .774 23.2 0 22.222 0h.003z"/></svg>`

	svgFacebookIcon = `<svg class="follow-us__icon" viewBox="0 0 24 24" width="28" height="28" aria-hidden="true"><path fill="currentColor" d="M24 12.073C24 5.446 18.627.073 12 .073S0 5.446 0 12.073c0 5.99 4.388 10.954 10.125 11.854v-8.385H7.078v-3.47h3.047V9.43c0-3.007 1.792-4.669 4.533-4.669 1.312 0 2.686.235 2.686.235v2.953H15.83c-1.491 0-1.956.925-1.956 1.874v2.25h3.328l-.532 3.47h-2.796v8.385C19.612 23.027 24 18.062 24 12.073z"/></svg>`
)

func buildFollowUsSection(content map[string]string, s SocialLinks) string {
	var buttons []string
	if u := strings.TrimSpace(s.Github); u != "" {
		buttons = append(buttons, fmt.Sprintf(
			`<a class="follow-us__btn follow-us__btn--github" href="%s"%s aria-label="GitHub">%s</a>`,
			html.EscapeString(u), externalLinkAttrs(u), svgGitHubIcon))
	}
	if u := strings.TrimSpace(s.Linkedin); u != "" {
		buttons = append(buttons, fmt.Sprintf(
			`<a class="follow-us__btn follow-us__btn--linkedin" href="%s"%s aria-label="LinkedIn">%s</a>`,
			html.EscapeString(u), externalLinkAttrs(u), svgLinkedInIcon))
	}
	if u := strings.TrimSpace(s.Facebook); u != "" {
		buttons = append(buttons, fmt.Sprintf(
			`<a class="follow-us__btn follow-us__btn--facebook" href="%s"%s aria-label="Facebook">%s</a>`,
			html.EscapeString(u), externalLinkAttrs(u), svgFacebookIcon))
	}

	title := strings.TrimSpace(content["follow_us_title"])
	hasButtons := len(buttons) > 0
	if title == "" && !hasButtons {
		return ""
	}
	if title == "" {
		title = "Follow us"
	}

	if !hasButtons {
		return fmt.Sprintf(
			`<section class="follow-us section section-gradient scroll-reveal" id="contact"><div class="container"><h2 id="follow_us_title" class="center">%s</h2></div></section>`,
			html.EscapeString(title))
	}

	return fmt.Sprintf(
		`<section class="follow-us section section-gradient scroll-reveal" id="contact"><div class="container"><h2 id="follow_us_title" class="center">%s</h2><div class="follow-us__buttons">%s</div></div></section>`,
		html.EscapeString(title),
		strings.Join(buttons, ""),
	)
}

func buildOffersItems(offers []OfferItem) string {
	if len(offers) == 0 {
		return ""
	}
	parts := make([]string, 0, len(offers))
	for _, o := range offers {
		image := strings.TrimSpace(o.Image)
		title := strings.TrimSpace(o.Title)
		text := strings.TrimSpace(o.Text)
		if image == "" && title == "" && text == "" {
			continue
		}
		var b strings.Builder
		b.WriteString(`<article class="offer-card">`)
		if image != "" {
			b.WriteString(fmt.Sprintf(`<img class="offer-card__image" src="%s" alt="%s">`, html.EscapeString(image), html.EscapeString(title)))
		}
		if title != "" {
			b.WriteString(fmt.Sprintf(`<h3>%s</h3>`, html.EscapeString(title)))
		}
		if text != "" {
			b.WriteString(fmt.Sprintf(`<p>%s</p>`, html.EscapeString(text)))
		}
		b.WriteString(`</article>`)
		parts = append(parts, b.String())
	}
	return strings.Join(parts, "\n")
}

func buildPhotosItems(photos []string) string {
	if len(photos) == 0 {
		return ""
	}

	parts := make([]string, 0, len(photos))
	for i, photo := range photos {
		part := fmt.Sprintf(`<img src="%s" alt="photo %d">`, html.EscapeString(photo), i+1)
		parts = append(parts, part)
	}
	return strings.Join(parts, "\n")
}

func buildSplitWidget(ariaLabel, idPrefix string, entries []splitEntry) string {
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

func buildVacanciesSection(vacancies []Vacancy, content map[string]string) string {
	if len(vacancies) == 0 {
		return ""
	}
	widget := buildVacanciesWidget(vacancies, content)
	title := strings.TrimSpace(content["vacancies_title"])
	return fmt.Sprintf(
		`<section class="vacancies section section-gradient scroll-reveal" id="vacancies"><div class="container"><h2 id="vacancies_title" class="center">%s</h2>%s</div></section>`,
		html.EscapeString(title),
		widget,
	)
}

func buildVacanciesWidget(vacancies []Vacancy, content map[string]string) string {
	if len(vacancies) == 0 {
		return ""
	}

	reqTitle := html.EscapeString(content["vacancy_requirements_title"])
	respTitle := html.EscapeString(content["vacancy_responsibilities_title"])
	advTitle := html.EscapeString(content["vacancy_advantages_title"])

	entries := make([]splitEntry, 0, len(vacancies))
	for _, vacancy := range vacancies {
		entries = append(entries, splitEntry{
			Label: vacancy.Role,
			Body:  buildVacancyDetailHTML(vacancy, reqTitle, respTitle, advTitle),
		})
	}
	return buildSplitWidget("Open positions", "vacancy", entries)
}

func buildVacancyDetailHTML(v Vacancy, reqTitle, respTitle, advTitle string) string {
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
	b.WriteString(`<div class="footer-contact-row">`)
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
