package main

import (
	"bytes"
	"io"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestStaticFileHandler(t *testing.T) {
	dir := t.TempDir()
	if err := os.WriteFile(filepath.Join(dir, "index.html"), []byte("<html>ok</html>"), 0o644); err != nil {
		t.Fatal(err)
	}

	srv := httptest.NewServer(staticFileHandler(dir))
	defer srv.Close()

	resp, err := http.Get(srv.URL + "/index.html")
	if err != nil {
		t.Fatal(err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		t.Fatalf("expected 200, got %d", resp.StatusCode)
	}
}

func TestRunCLIServeWithValidateRejected(t *testing.T) {
	var stderr bytes.Buffer
	code := runCLI([]string{"--validate", "--serve", "--site", "content/kometa"}, nil, io.Discard, &stderr)
	if code != 2 {
		t.Fatalf("expected exit 2, got %d stderr=%q", code, stderr.String())
	}
	if !strings.Contains(stderr.String(), "cannot use --serve with --validate") {
		t.Fatalf("unexpected stderr: %q", stderr.String())
	}
}
