.POSIX:

DEBUG_APK = target/android-artifacts/debug/apk/fagman-app.apk

default: apk

apk:
	podman run -v $(shell pwd)/:/root/fagman -w /root/fagman/ -t apk cargo quad-apk build
	$(MAKE) install-apk

install-apk:
	-mv $(DEBUG_APK) .
	-adb uninstall fagman.fagman_app
	adb install -r fagman-app.apk
	reset
	adb logcat -c
	adb shell monkey -p fagman.fagman_app -c android.intent.category.LAUNCHER 1
	adb logcat -v color -s fagman -s SAPP -s libc -s DEBUG -s ActivityManager -s ActivityTaskManager -s WindowManager -s AndroidRuntime -s rkfi.fagman_app | tee output.log

# Useful for dev
cli:
	podman run -v $(shell pwd)/:/root/fagman -w /root/fagman/ -it apk bash

clean:
	podman run -v $(shell pwd):/root/fagman -w /root/fagman -t apk rm -fr target/

.PHONY: apk install-apk cli clean
