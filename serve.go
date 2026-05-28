package main

import (
	"context"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"
)

const defaultServePort = 8080

func staticFileHandler(dir string) http.Handler {
	return http.FileServer(http.Dir(dir))
}

func serveStaticDir(dir string, port int, stdout io.Writer) error {
	addr := fmt.Sprintf("127.0.0.1:%d", port)
	srv := &http.Server{
		Addr:    addr,
		Handler: staticFileHandler(dir),
	}

	stop := make(chan os.Signal, 1)
	signal.Notify(stop, os.Interrupt, syscall.SIGTERM)
	defer signal.Stop(stop)

	go func() {
		<-stop
		ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
		defer cancel()
		_ = srv.Shutdown(ctx)
	}()

	fmt.Fprintf(stdout, "Serving %s at http://%s/\n", dir, addr)
	fmt.Fprintln(stdout, "Press Ctrl+C to stop.")

	err := srv.ListenAndServe()
	if err == http.ErrServerClosed {
		return nil
	}
	return err
}
