package main

import (
	"strings"
	"testing"
)

func TestBuildFooterOuterHTMLNoScrollReveal(t *testing.T) {
	enabled := true
	html := buildFooterOuterHTML(FooterConfig{
		Enabled:      &enabled,
		SectionTitle: "Contacts",
		Contact: FooterContact{
			Email: "hello@example.com",
		},
	})
	if html == "" {
		t.Fatal("expected footer HTML")
	}
	if strings.Contains(html, "scroll-reveal") {
		t.Fatalf("footer must not use scroll-reveal, got: %s", html)
	}
	if !strings.Contains(html, `id="footer"`) {
		t.Fatalf("expected footer id, got: %s", html)
	}
	if !strings.Contains(html, `id="footer-contact"`) {
		t.Fatalf("expected footer contact id, got: %s", html)
	}
}

func TestBuildFooterLegalRowOmitsEmptyTermsURL(t *testing.T) {
	row := buildFooterLegalRow(FooterConfig{
		PrivacyURL:   "https://example.com/privacy",
		PrivacyLabel: "Privacy Policy",
		TermsURL:     "",
		TermsLabel:   "Terms of Service",
	})
	if !strings.Contains(row, "Privacy Policy") {
		t.Fatalf("expected privacy link, got: %s", row)
	}
	if strings.Contains(row, "Terms of Service") {
		t.Fatalf("expected no terms link when URL empty, got: %s", row)
	}
}
