package main

/*
#include <stdlib.h>
*/
import "C"
import (
	"context"
	"fmt"
	"github.com/sagernet/sing-box/option"
	"github.com/sagernet/sing-box"
)

var (
	currentBox *box.Box
	cancelFunc context.CancelFunc
)

//export sing_box_start
func sing_box_start(configJson *C.char) C.int {
	if currentBox != nil {
		return -1
	}
	jsonStr := C.GoString(configJson)
	ctx, cancel := context.WithCancel(context.Background())
	cancelFunc = cancel
	var opts option.Options
	if err := opts.UnmarshalJSON([]byte(jsonStr)); err != nil {
		fmt.Printf("Failed to parse config: %v\n", err)
		return -2
	}
	b, err := box.New(box.Options{
		Context: ctx,
		Options: opts,
	})
	if err != nil {
		fmt.Printf("Failed to create box: %v\n", err)
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
	if cancelFunc != nil {
		cancelFunc()
	}
	currentBox = nil
	return 0
}

func main() {}
