package main

import (
	"crypto/md5"
	"fmt"
	"net/http"
	"os/exec"
)

// Intentionally vulnerable demo for Bugbee (DO NOT deploy)

func search(w http.ResponseWriter, r *http.Request) {
	q := r.URL.Query().Get("q")
	// SQL injection style concat
	query := "SELECT * FROM users WHERE name = '" + q + "'"
	fmt.Fprintln(w, query)
}

func run(w http.ResponseWriter, r *http.Request) {
	cmd := r.FormValue("cmd")
	out, _ := exec.Command("sh", "-c", cmd).CombinedOutput()
	w.Write(out)
}

func hashPassword(p string) string {
	sum := md5.Sum([]byte(p))
	return fmt.Sprintf("%x", sum)
}

func main() {
	http.HandleFunc("/search", search)
	http.HandleFunc("/run", run)
	http.ListenAndServe(":8080", nil)
}
