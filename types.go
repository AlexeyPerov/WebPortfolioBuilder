package main

import "strings"

type Config struct {
	OutputFolder   string             `json:"output_folder"`
	Theme          map[string]string  `json:"theme"`
	Content        map[string]string  `json:"content"`
	Offers         []OfferItem        `json:"offers"`
	Photos         []string           `json:"photos"`
	GameStoreIcons GameStoreIcons     `json:"game_store_icons"`
	GameSubscribe  GameSubscribeBlock `json:"game_subscribe"`
	Games          []Game             `json:"games"`
	Vacancies      []Vacancy          `json:"vacancies"`
	Social         SocialLinks        `json:"social"`
	Footer         FooterConfig       `json:"footer"`
}

// OfferItem is one card in the "We offer" section (any number of items).
type OfferItem struct {
	Image string `json:"image"`
	Title string `json:"title"`
	Text  string `json:"text"`
}

// GameStoreIcons holds image paths used for all game cards’ store buttons (shared across games).
type GameStoreIcons struct {
	GooglePlay string `json:"google_play"`
	AppStore   string `json:"app_store"`
	Galaxy     string `json:"galaxy"`
	Amazon     string `json:"amazon"`
}

// GameSubscribeBlock is shown inside each game card above store buttons when at least one link has a URL.
type GameSubscribeBlock struct {
	Title string              `json:"title"`
	Links []GameSubscribeLink `json:"links"`
}

type GameSubscribeLink struct {
	Label string `json:"label"`
	URL   string `json:"url"`
}

func (i GameStoreIcons) withDefaults() GameStoreIcons {
	pick := func(s, fallback string) string {
		if strings.TrimSpace(s) != "" {
			return strings.TrimSpace(s)
		}
		return fallback
	}
	return GameStoreIcons{
		GooglePlay: pick(i.GooglePlay, "Images/gp-store-icon.png"),
		AppStore:   pick(i.AppStore, "Images/appstore-store-icon.png"),
		Galaxy:     pick(i.Galaxy, "Images/galaxy-store-icon.png"),
		Amazon:     pick(i.Amazon, "Images/amazon-store-icon.png"),
	}
}

type SocialLinks struct {
	Github   string `json:"github_url"`
	Linkedin string `json:"linkedin_url"`
	Facebook string `json:"facebook_url"`
}

type Game struct {
	Image          string   `json:"image"`
	HeaderImage    string   `json:"header_image"`
	SwiperImages   []string `json:"swiper_images"`
	CardBackground string   `json:"card_background"`
	Title          string   `json:"title"`
	Text1          string   `json:"text_1"`
	Text2          string   `json:"text_2"`
	StatLeftLine1  string   `json:"stat_left_line_1"`
	StatLeftLine2  string   `json:"stat_left_line_2"`
	StatRightLine1 string   `json:"stat_right_line_1"`
	StatRightLine2 string   `json:"stat_right_line_2"`
	GooglePlayURL  string   `json:"google_play_url"`
	AppStoreURL    string   `json:"app_store_url"`
	AmazonStoreURL string   `json:"amazon_store_url"`
	GalaxyStoreURL string   `json:"galaxy_store_url"`
}

type Vacancy struct {
	Role             string   `json:"role"`
	Requirements     []string `json:"requirements"`
	Responsibilities []string `json:"responsibilities"`
	Advantages       []string `json:"advantages"`
	ApplyURL         string   `json:"apply_url"`
	ApplyLabel       string   `json:"apply_label"`
}

type FooterConfig struct {
	SectionTitle string        `json:"section_title"`
	Contact      FooterContact `json:"contact"`
	PrivacyURL   string        `json:"privacy_url"`
	PrivacyLabel string        `json:"privacy_label"`
	TermsURL     string        `json:"terms_url"`
	TermsLabel   string        `json:"terms_label"`
	Copyright    string        `json:"copyright"`
	CookieNotice string        `json:"cookie_notice"`
}

type FooterContact struct {
	Paragraphs []string `json:"paragraphs"`
	Email      string   `json:"email"`
}

type splitEntry struct {
	Label string
	Body  string
}
