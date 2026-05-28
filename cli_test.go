package main

import (
	"bytes"
	"io"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestDiscoverContentBundles(t *testing.T) {
	root := t.TempDir()
	contentDir := filepath.Join(root, "content")
	if err := os.MkdirAll(filepath.Join(contentDir, "alpha", "pages"), 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.MkdirAll(filepath.Join(contentDir, "beta", "pages"), 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.MkdirAll(filepath.Join(contentDir, "assets-only"), 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.MkdirAll(filepath.Join(contentDir, "broken"), 0o755); err != nil {
		t.Fatal(err)
	}
	writeJSONFile(t, filepath.Join(contentDir, "alpha", "site.json"), `{"site_id":"alpha","output_folder":"Results/Alpha"}`)
	writeJSONFile(t, filepath.Join(contentDir, "beta", "site.json"), `{"site_id":"beta","output_folder":"Results/Beta"}`)
	writeJSONFile(t, filepath.Join(contentDir, "broken", "site.json"), `{not json`)

	got, err := discoverContentBundles(root)
	if err != nil {
		t.Fatalf("discoverContentBundles: %v", err)
	}
	if len(got) != 2 || got[0] != "content/alpha" || got[1] != "content/beta" {
		t.Fatalf("unexpected bundles: %#v", got)
	}
}

func TestResolveBundleChoice(t *testing.T) {
	bundles := []string{"content/kometa", "content/my-studio"}
	if got, ok := resolveBundleChoice("2", bundles); !ok || got != "content/my-studio" {
		t.Fatalf("expected numbered choice, got %q ok=%v", got, ok)
	}
	if _, ok := resolveBundleChoice("9", bundles); ok {
		t.Fatal("expected out-of-range choice to fail")
	}
}

func TestValidateSiteRejectsUnknownWidgetType(t *testing.T) {
	siteDir := createTestSiteDir(t)
	writeJSONFile(t, filepath.Join(siteDir, "site.json"), `{"site_id":"test","output_folder":"Results/Test"}`)
	writeJSONFile(t, filepath.Join(siteDir, "pages", "home.json"), `{"slug":"","widgets":[{"type":"not_a_widget","props":{}}]}`)

	bundle, _, err := loadSiteBundle(siteDir)
	if err != nil {
		t.Fatalf("loadSiteBundle: %v", err)
	}

	_, err = validateSiteBundleOnly(bundle, "Template")
	if err == nil {
		t.Fatal("expected unknown widget type error")
	}
	if !strings.Contains(err.Error(), `unknown widget type "not_a_widget"`) {
		t.Fatalf("unexpected error: %v", err)
	}
}

func TestRunCLIValidateKometa(t *testing.T) {
	if _, err := os.Stat(filepath.Join("content", "kometa", "site.json")); err != nil {
		t.Skip("kometa bundle not present")
	}

	var stdout, stderr bytes.Buffer
	code := runCLI([]string{"--validate", "--site", "content/kometa"}, bytes.NewReader(nil), &stdout, &stderr)
	if code != 0 {
		t.Fatalf("expected exit 0, got %d stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stdout.String(), "Validation passed:") {
		t.Fatalf("expected validation success message, got stdout=%q", stdout.String())
	}
}

func TestRunCLIListSites(t *testing.T) {
	if _, err := os.Stat(filepath.Join("content", "kometa", "site.json")); err != nil {
		t.Skip("kometa bundle not present")
	}

	var stdout bytes.Buffer
	code := runCLI([]string{"--list-sites"}, bytes.NewReader(nil), &stdout, io.Discard)
	if code != 0 {
		t.Fatal("expected exit 0")
	}
	if !strings.Contains(stdout.String(), "content/kometa") {
		t.Fatalf("expected kometa in list, got %q", stdout.String())
	}
}

func TestRunCLINonInteractiveBuild(t *testing.T) {
	root := t.TempDir()
	templateDir := filepath.Join(root, "Template")
	if err := os.MkdirAll(templateDir, 0o755); err != nil {
		t.Fatal(err)
	}
	copyTestTemplateTree(t, "Template", templateDir)

	siteDir := filepath.Join(root, "content", "mini")
	pagesDir := filepath.Join(siteDir, "pages")
	if err := os.MkdirAll(pagesDir, 0o755); err != nil {
		t.Fatal(err)
	}
	writeJSONFile(t, filepath.Join(siteDir, "site.json"), `{"site_id":"mini","output_folder":"Results/Mini"}`)
	writeJSONFile(t, filepath.Join(pagesDir, "home.json"), `{"slug":"","widgets":[{"type":"intro","props":{"title":"Hi"}}]}`)

	t.Chdir(root)

	var stdout bytes.Buffer
	code := runCLI([]string{"--site", "content/mini"}, bytes.NewReader(nil), &stdout, io.Discard)
	if code != 0 {
		t.Fatalf("expected exit 0, got build failure")
	}
	if _, err := os.Stat(filepath.Join(root, "Results", "Mini", "index.html")); err != nil {
		t.Fatalf("expected generated index.html: %v", err)
	}
}

func copyTestTemplateTree(t *testing.T, srcRoot, dstRoot string) {
	t.Helper()
	err := filepath.WalkDir(srcRoot, func(path string, d os.DirEntry, walkErr error) error {
		if walkErr != nil {
			return walkErr
		}
		rel, err := filepath.Rel(srcRoot, path)
		if err != nil {
			return err
		}
		if rel == "." {
			return nil
		}
		target := filepath.Join(dstRoot, rel)
		if d.IsDir() {
			return os.MkdirAll(target, 0o755)
		}
		data, err := os.ReadFile(path)
		if err != nil {
			return err
		}
		return os.WriteFile(target, data, 0o644)
	})
	if err != nil {
		t.Fatalf("copy template tree: %v", err)
	}
}
