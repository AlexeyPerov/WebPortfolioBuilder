package main

import "testing"

func TestBuildRouteIndexMapsHomeAndSlug(t *testing.T) {
	bundle := SiteBundle{
		Pages: []SitePageFile{
			{Path: "pages/home.json", Page: PageConfig{Slug: ""}},
			{Path: "pages/about.json", Page: PageConfig{Slug: "about"}},
		},
	}

	index, err := buildRouteIndex(bundle)
	if err != nil {
		t.Fatalf("buildRouteIndex returned error: %v", err)
	}

	home := index.BySlug[""]
	if home.OutputRelPath != "index.html" {
		t.Fatalf("expected home output index.html, got %q", home.OutputRelPath)
	}

	about := index.BySlug["about"]
	if about.OutputRelPath != "about/index.html" {
		t.Fatalf("expected about output about/index.html, got %q", about.OutputRelPath)
	}
}

func TestResolveInternalSlugReferenceUsesRelativePaths(t *testing.T) {
	routes := map[string]PageRoute{
		"":      {Slug: "", DirRelPath: ""},
		"about": {Slug: "about", DirRelPath: "about"},
	}

	fromHome, err := resolveInternalSlugReference(routes[""], "about", routes)
	if err != nil {
		t.Fatalf("unexpected error from home: %v", err)
	}
	if fromHome != "about/" {
		t.Fatalf("expected home->about to be about/, got %q", fromHome)
	}

	fromAbout, err := resolveInternalSlugReference(routes["about"], "", routes)
	if err != nil {
		t.Fatalf("unexpected error from about: %v", err)
	}
	if fromAbout != "../" {
		t.Fatalf("expected about->home to be ../, got %q", fromAbout)
	}
}

func TestResolveNavHrefPrefixesHashLinksFromNestedPage(t *testing.T) {
	routes := map[string]PageRoute{
		"":      {Slug: "", DirRelPath: ""},
		"about": {Slug: "about", DirRelPath: "about"},
	}

	fromHome, err := resolveNavHref(routes[""], "#intro_title", routes)
	if err != nil {
		t.Fatalf("unexpected error from home: %v", err)
	}
	if fromHome != "#intro_title" {
		t.Fatalf("expected home hash to stay %q, got %q", "#intro_title", fromHome)
	}

	fromAbout, err := resolveNavHref(routes["about"], "#intro_title", routes)
	if err != nil {
		t.Fatalf("unexpected error from about: %v", err)
	}
	if fromAbout != "../#intro_title" {
		t.Fatalf("expected about hash to be ../#intro_title, got %q", fromAbout)
	}
}

func TestResolveNavHrefEmptyHrefIsHome(t *testing.T) {
	routes := map[string]PageRoute{
		"":      {Slug: "", DirRelPath: ""},
		"about": {Slug: "about", DirRelPath: "about"},
	}

	fromAbout, err := resolveNavHref(routes["about"], "", routes)
	if err != nil {
		t.Fatalf("unexpected error: %v", err)
	}
	if fromAbout != "../" {
		t.Fatalf("expected empty href from about to be ../, got %q", fromAbout)
	}
}

func TestResolveInternalSlugReferenceRejectsUnknownSlug(t *testing.T) {
	routes := map[string]PageRoute{
		"": {Slug: "", DirRelPath: ""},
	}

	if _, err := resolveInternalSlugReference(routes[""], "missing", routes); err == nil {
		t.Fatal("expected unknown slug error")
	}
}
