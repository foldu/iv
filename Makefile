DESTDIR=${HOME}/.local

all:
	cargo build --release

install:
	mkdir -p $(DESTDIR)/share/applications $(DESTDIR)/bin
	cp iv.desktop $(DESTDIR)/share/applications
	install -D target/release/iv $(DESTDIR)/bin

uninstall:
	rm $(DESTDIR)/share/applications/iv.desktop $(DESTDIR)/bin/iv
