package main

/*
#include <stdlib.h>
*/
import "C"
import (
	"context"
	"fmt"
	"sync"

	"github.com/sagernet/sing-box"
	"github.com/sagernet/sing-box/include"
	"github.com/sagernet/sing-box/option"
)

var (
	currentBox *box.Box
	logMu      sync.Mutex
	logBuffer  []string
	maxLogs    = 500
)

type logWriter struct{}

func (w *logWriter) Write(p []byte) (n int, err error) {
	logMu.Lock()
	defer logMu.Unlock()
	
	msg := string(p)
	logBuffer = append(logBuffer, msg)
	if len(logBuffer) > maxLogs {
		logBuffer = logBuffer[len(logBuffer)-maxLogs:]
	}
	return len(p), nil
}

//export sing_box_start
func sing_box_start(configJson *C.char) C.int {
	if currentBox != nil {
		return -1
	}
	jsonStr := C.GoString(configJson)
	
	ctx := include.Context(context.Background())

	var opts option.Options
	if err := opts.UnmarshalJSONContext(ctx, []byte(jsonStr)); err != nil {
		fmt.Printf("Failed to parse config: %v\nRaw JSON: %s\n", err, jsonStr)
		return -2
	}

	b, err := box.New(box.Options{
		Context:           ctx,
		Options:           opts,
		PlatformLogWriter: &logWriter{},
	})
	if err != nil {
		fmt.Printf("Failed to create box: %v\nRaw JSON: %s\n", err, jsonStr)
		return -3
	}
	if err := b.Start(); err != nil {
		fmt.Printf("Failed to start box: %v\n", err)
		return -4
	}
	currentBox = b
	return 0
}

//export sing_box_stop
func sing_box_stop() C.int {
	if currentBox == nil {
		return 0
	}
	if err := currentBox.Close(); err != nil {
		fmt.Printf("Failed to close box: %v\n", err)
		return -1
	}
	currentBox = nil
	return 0
}

//export sing_box_get_logs
func sing_box_get_logs() *C.char {
	logMu.Lock()
	defer logMu.Unlock()
	
	if len(logBuffer) == 0 {
		return C.CString("")
	}
	
	var combined string
	for _, l := range logBuffer {
		combined += l
	}
	// 清空以便下次增量获取（或者保留全量）
	// 这里我们选择全量获取但限制条数
	return C.CString(combined)
}

func main() {}
