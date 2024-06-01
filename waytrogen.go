package main

import (
	"bytes"
	"database/sql"
	"encoding/base64"
	"errors"
	"fmt"
	"github.com/disintegration/imaging"
	_ "github.com/mattn/go-sqlite3"
	"io/fs"
	"net/http"
	"os"
	"path/filepath"
)

type dbImage struct {
	mimeType  string
	base64Rep string
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
	stmt, err := tx.Prepare("INSERT INTO image(path, image_type, base64, date_modified) VALUES(?,?,?,?)")
	if err != nil {
		return
	}

	files, err := readFiles("/home/uwu/Downloads/Wallpapers/2024")
	if err != nil {
		return
	}
	images := getImages(files)
	for _, file := range images {
		fmt.Println(file.absPath)
		image, err := getImage(file)
		if err != nil {
			continue
		}
		info, err := file.dirEntry.Info()
		if err != nil {
			continue
		}
		stmt.Exec(file.absPath, image.mimeType, image.base64Rep, info.ModTime())
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

func getImage(file file) (dbImage, error) {
	f, err := os.Open(file.absPath)
	if err != nil {
		return dbImage{"", ""}, err
	}
	defer f.Close()
	fileTypeBuff := make([]byte, 512)
	if _, err := f.Read(fileTypeBuff); err != nil {
		return dbImage{"", ""}, err
	}
	fileType := http.DetectContentType(fileTypeBuff)
	if !validImage(fileType) {
		return dbImage{"", ""}, errors.New("File is not an image.")
	}
	image, err := imaging.Open(file.absPath)
	if err != nil {
		return dbImage{"", ""}, err
	}
	thumbnail := imaging.Thumbnail(image, 300, 300, imaging.Box)
	var imageBuff bytes.Buffer
	switch fileType {
	case "image/png":
		imaging.Encode(&imageBuff, thumbnail, imaging.PNG)
	case "image/jpeg":
		imaging.Encode(&imageBuff, thumbnail, imaging.JPEG)
	}
	return dbImage{fileType, base64.StdEncoding.EncodeToString(imageBuff.Bytes())}, nil
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
