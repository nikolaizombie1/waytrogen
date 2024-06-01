package main

import (
	"bytes"
	"database/sql"
	"encoding/base64"
	"io/fs"
	"net/http"
	"os"
	"path/filepath"
	"runtime"

	"github.com/disintegration/imaging"
	_ "github.com/mattn/go-sqlite3"
)

type dbImage struct {
	path          string
	mimeType      string
	base64Rep     string
	date_modified string
}

func main() {
	db, err := sql.Open("sqlite3", "./test.db")
	if err != nil {
		return
	}
	defer db.Close()
	sqlStmt := `CREATE TABLE IF NOT EXISTS image(path string, image_type string, base64 string, date_modified numeric);`
	db.Exec(sqlStmt)
	tx, err := db.Begin()
	if err != nil {
		return
	}
	insertStmt, err := tx.Prepare("INSERT INTO image(path, image_type, base64, date_modified) VALUES(?,?,?,?)")
	if err != nil {
		return
	}

	files, err := readFiles("/home/uwu/Downloads/Wallpapers/")
	if err != nil {
		return
	}
	images := getImages(files)
	uncached_images := []file{}
	for _, file := range images {
		selectStmt, err := db.Prepare("select path from image where path = ?;")
		if err != nil {
			continue
		}
		defer selectStmt.Close()
		var path string
		row := selectStmt.QueryRow(file.absPath)
		err = row.Scan(&path)
		if err == nil {
			continue
		}

		if path != "" {
			continue
		}
		uncached_images = append(uncached_images, file)
	}
	proccesed_images := []dbImage{}
	channel := make(chan dbImage)
	for i := 0; i < len(uncached_images); i+=runtime.NumCPU() {
		
		if len(uncached_images) - i < runtime.NumCPU() {
			for j := i; j < len(uncached_images); j++ {
				go getImage(uncached_images[i], channel)
			}
			for j := i; j < len(uncached_images); j++ {
				image := <-channel
				proccesed_images = append(proccesed_images, image)
			}
		} else {
			for j := i; j < i + runtime.NumCPU(); j++ {
				go getImage(uncached_images[i], channel)
			}
			for j := i; j < i + runtime.NumCPU(); j++ {
				image := <-channel
				proccesed_images = append(proccesed_images, image)
			}
		}
	}
	for _, image := range proccesed_images {
		insertStmt.Exec(image.path, image.mimeType, image.base64Rep, image.date_modified)
	}
	err = tx.Commit()
	if err != nil {
		return
	}
}

func getImages(files []file) []file {
	fileTypeBuff := make([]byte, 512)
	images := []file{}
	for _, f := range files {
		file, _ := os.Open(f.absPath)
		defer file.Close()
		if _, err := file.Read(fileTypeBuff); err != nil {
			continue
		}
		fileType := http.DetectContentType(fileTypeBuff)
		if !validImage(fileType) {
			continue
		}
		images = append(images, f)

	}
	return images
}

func getImage(file file, channel chan dbImage) {
	f, err := os.Open(file.absPath)
	if err != nil {
		return
	}
	defer f.Close()
	fileTypeBuff := make([]byte, 512)
	if _, err := f.Read(fileTypeBuff); err != nil {
		return
	}
	fileType := http.DetectContentType(fileTypeBuff)
	if !validImage(fileType) {
		return
	}
	image, err := imaging.Open(file.absPath)
	if err != nil {
		return
	}
	thumbnail := imaging.Thumbnail(image, 300, 300, imaging.Box)
	var imageBuff bytes.Buffer
	switch fileType {
	case "image/png":
		imaging.Encode(&imageBuff, thumbnail, imaging.PNG)
	case "image/jpeg":
		imaging.Encode(&imageBuff, thumbnail, imaging.JPEG)
	}
	info, err := file.dirEntry.Info()
	if err != nil {
		return
	}
	channel <- dbImage{file.absPath, fileType, base64.StdEncoding.EncodeToString(imageBuff.Bytes()), info.ModTime().GoString()}
}

func maxImageSize(files []*os.File) (int64, error) {
	var maxSize int64 = 0
	for _, file := range files {
		stat, err := file.Stat()
		if err != nil {
			return 0, err
		}
		size := stat.Size()
		if size > maxSize {
			maxSize = size
		}
	}
	return maxSize, nil
}

func validImage(mimeFile string) bool {
	switch mimeFile {
	case "image/png":
		return true
	case "image/jpeg":
		return true
	default:
		return false
	}
}

func readFiles(path string) ([]file, error) {
	return readDir([]file{}, path)
}

type file struct {
	dirEntry fs.DirEntry
	absPath  string
}

func readDir(files []file, path string) ([]file, error) {
	dir, err := os.ReadDir(path)
	if err != nil {
		return files, err
	}

	for _, entry := range dir {
		absPath := filepath.Join(path, entry.Name())
		if entry.IsDir() {
			files, err = readDir(files, absPath)
			if err != nil {
				return files, err
			}

		} else {
			files = append(files, file{entry, absPath})
		}
	}
	return files, nil
}
