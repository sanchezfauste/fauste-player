# Pierwsze kroki

## Instalacja {#install}

Pobierz pakiet dla swojego systemu ze strony **Releases** projektu. Obok
każdego pliku leży plik `.sha256`. Aby sprawdzić pobrany plik, uruchom
`sha256sum -c <file>.sha256` w Linuksie lub `shasum -a 256 -c <file>.sha256`
w macOS.

### Linux {#linux}

| Pakiet | Instalacja | Aktualizacja | Usunięcie |
|---|---|---|---|
| `.deb` (Debian, Ubuntu) | `sudo apt install ./fauste-player_<version>_amd64.deb` | zainstaluj nowszy `.deb` w ten sam sposób | `sudo apt remove fauste-player` |
| `.rpm` (Fedora, openSUSE) | `sudo dnf install ./fauste-player-<version>-1.x86_64.rpm` | zainstaluj nowszy `.rpm` | `sudo dnf remove fauste-player` |
| AppImage (dowolna dystrybucja) | `chmod +x fauste-player-<version>-x86_64.AppImage`, a następnie uruchom plik | zastąp plik | usuń plik |
| Pakiet Flatpak | `flatpak install --user fauste-player-<version>-x86_64.flatpak` | zainstaluj nowszy pakiet | `flatpak uninstall org.fauste.FaustePlayer` |
| Archiwum | rozpakuj je i uruchom `fauste-player` | rozpakuj nowsze | usuń folder |

- **Integracja z pulpitem:** pakiety dodają **Fauste Player** do menu
  aplikacji i pozwalają otwierać nim playlisty `.m3u`, `.m3u8` i `.pls`,
  które są wtedy importowane jako nowe playlisty.
- **Wymagane biblioteki:** biblioteki ALSA i D-Bus. Mają je wszystkie
  środowiska graficzne, a pakiety je deklarują. JACK jest używany, gdy jest
  zainstalowany, i nigdy nie jest wymagany.
- **AppImage:** jeśli nie uruchamia się z powodu braku FUSE, uruchom go z
  opcją `--appimage-extract-and-run`.
- **Flatpak:**
  - Pakiet wymaga środowiska uruchomieniowego freedesktop z Flathub. Jeśli
    zdalne repozytorium Flathub nie jest skonfigurowane, najpierw uruchom
    `flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo`.
  - Piaskownica może czytać Twój folder domowy (aby odtwarzać muzykę tam,
    gdzie leży).
  - Gra przez PulseAudio, a w przypadku urządzeń bit-perfect bezpośrednio
    przez ALSA.
- **Biblioteki pulpitu:** okno korzysta z libxkbcommon oraz EGL lub OpenGL,
  z Waylandem lub X11. Mają je wszystkie środowiska graficzne, a pakiety
  `.deb` i `.rpm` je deklarują. W bardzo minimalnym systemie zainstaluj je
  przed użyciem AppImage.

### Windows {#windows}

Uruchom `fauste-player-<version>-x86_64-pc-windows-msvc.msi`. Instaluje się
dla wszystkich użytkowników w *Program Files* i dodaje wpis w menu Start.
- **Aktualizacja:** uruchom nowszy instalator. Zastąpi zainstalowaną wersję.
- **Usunięcie:** użyj **Ustawienia → Aplikacje**.
- **Niepodpisany instalator:** jeśli wydanie nie jest podpisane, SmartScreen
  ostrzega o nieznanym wydawcy. Wybierz **Więcej informacji → Uruchom mimo
  to**.

Archiwum `.zip` to przenośna alternatywa: rozpakuj je i uruchom
`fauste-player.exe`.

### macOS {#macos}

Otwórz `fauste-player-<version>-macos-universal.dmg` i przeciągnij **Fauste
Player** do **Programów**. Ta sama aplikacja działa na Apple silicon i na
Intelu (macOS 11 lub nowszy).
- **Aktualizacja:** zastąp aplikację w ten sam sposób.
- **Usunięcie:** przenieś ją do Kosza.
- **Niepodpisana aplikacja:** jeśli wydanie nie jest podpisane i
  notaryzowane, macOS odmawia pierwszego uruchomienia.
  - macOS 15 i nowszy: otwórz **Ustawienia systemowe → Prywatność i
    ochrona**, przewiń do komunikatu o Fauste Player, wybierz **Otwórz
    mimo to** i potwierdź.
  - macOS 14 i starszy: kliknij aplikację prawym przyciskiem, wybierz
    **Otwórz**, a następnie potwierdź.
  - Albo uruchom `xattr -dr com.apple.quarantine "/Applications/Fauste Player.app"`.
- **JACK w macOS:** podpisana i notaryzowana aplikacja może załadować tylko
  taką bibliotekę JACK, która sama jest podpisana. W przeciwnym razie JACK
  jest pokazywany jako niedostępny; użyj Core Audio.
- **Playlisty:** w macOS importuje się je w Ustawienia → Playlisty. Otwieranie
  pliku playlisty aplikacją z Findera nie jest obsługiwane.

### Jedna instancja naraz {#one-instance-at-a-time}

Na jeden folder danych działa tylko jeden Fauste Player. Otwarcie playlisty
z menedżera plików, gdy program działa, importuje tę playlistę do
uruchomionej aplikacji. Ponowne uruchomienie bez playlisty pokazuje
komunikat, że program już działa. Aby uruchomić osobne instancje obok siebie
(na przykład dwa studia na jednym komputerze), nadaj każdej własny folder za
pomocą `FAUSTE_HOME`.

### Wiersz poleceń {#command-line}

`fauste-player --version` wypisuje wersję. Pasek tytułu okna pokazuje nazwę
i wersję, a przycisk **O programie** (ikona informacji, na lewo od
**Ustawień**) otwiera okno **O programie** z informacją o prawach autorskich
i licencjach (**Licencje stron trzecich** otwiera plik z informacjami
instalowany z pakietami wydań). W języku przetłumaczonym za pomocą AI okno
to mówi też, że tłumaczenie może zawierać błędy. Zamkniesz je przyciskiem
**Zamknij** lub klawiszem `Esc`. `fauste-player --help` wyświetla listę
opcji. Pliki playlist podane jako argumenty są importowane jako nowe
playlisty.

![Okno O programie: wersja, prawa autorskie i licencje dołączonych komponentów](../../images/guide/about.png)

W Windows program wydania nie otwiera okna konsoli: `--version`, `--help` i
błędy uruchamiania pojawiają się zamiast tego w oknie komunikatu.

Gdy ustawienie wymaga ponownego uruchomienia, na górnym pasku pojawia się
plakietka **Wymagany restart**: naciśnij ją, aby uruchomić program ponownie
(zob. [Ustawienia](settings.md#restart-pending)).

### Zamykanie, gdy dźwięk jest na antenie {#closing-while-audio-is-on-air}

Zamknięcie okna, gdy coś jest na antenie, nie kończy programu. Okno wychodzi
na wierzch (nawet jeśli było zminimalizowane), a okno dialogowe **Dźwięk jest
na antenie** wymienia to, co brzmi: odtwarzacze, które grają lub są
wstrzymane (`P1 — tytuł`), oraz grające carty z ich numerem na stronie
(`Cart 3 — tytuł`). CUE odtwarzacza lub cartwalla się nie liczy. Wybierz
**Anuluj** (albo `Esc`, albo kliknij poza oknem dialogowym), aby grać dalej,
lub **Zatrzymaj i zamknij**, aby zatrzymać każdy odtwarzacz na antenie i
wszystkie carty, a następnie zakończyć program. Sesja jest zapisywana jak
przy każdym innym zakończeniu. Gdy nic nie jest na antenie, okno zamyka się
od razu. Okno dialogowe ma pierwszeństwo przed Ustawieniami i oknem O
programie, a skróty klawiszowe nic nie robią, dopóki jest otwarte (polecenia
MIDI i zdalne nadal działają).

## Pierwsze uruchomienie {#first-start}

Okno otwiera się z czterema odtwarzaczami, z których każdy pokazuje pustą
playlistę. Nic nie gra, dopóki nie naciśniesz Play: dotyczy to także
ponownego uruchomienia lub awarii.

## Dodawanie muzyki {#add-music}

- Kliknij **+ Dodaj** na dole odtwarzacza i wybierz pliki, albo
- przeciągnij pliki audio lub foldery z menedżera plików na listę utworów.
  Foldery dodają pliki audio znajdujące się bezpośrednio w nich, nie w ich
  podfolderach.

Obsługiwane formaty: WAV, AIFF, CAF, FLAC, MP3 (oraz MP1/MP2), AAC/M4A, ALAC, Ogg Vorbis, Opus, WavPack, Monkey's Audio (APE), DSD (DSF i DFF) oraz dźwięk Matroska (MKA).
Pliki DSD są odtwarzane po konwersji do PCM z częstotliwością równą ich
częstotliwości podzielonej przez 32 (88,2 kHz dla DSD64) albo bez zmian jako
DoP lub natywne DSD na urządzeniu bit-perfect ustawionym w ten sposób (zob.
[Wyjście bit-perfect](bit-perfect.md#dsd)). Opus zawsze gra z częstotliwością
48 kHz. Pliki WavPack muszą być bezstratne, mono lub stereo; hybrydowe i
wielokanałowe pliki WavPack są pokazywane jako nieczytelne, podobnie jak
pliki DFF skompresowane DST.

Każdy plik jest analizowany w tle. Analiza odczytuje tytuł, wykonawcę, album
i okładkę, rysuje przebieg oraz wyznacza, gdzie dźwięk się zaczyna i kończy
oraz gdzie wejść miksem w następny utwór. Utwór możesz odtworzyć, zanim jego
analiza się zakończy.

## Odtwarzanie {#play}

- Naciśnij **Play** (albo klawisz numeryczny odtwarzacza, `1` dla P1), aby
  uruchomić **następny** utwór, oznaczony na liście na zielono.
- Naciśnij **Play** ponownie, gdy utwór jest na antenie, aby przejść
  płynnie do następnego.
- **Kliknij dwukrotnie** utwór, aby ustawić go jako następny.

W trybie **CONT** (ciągłym) odtwarzacz sam miksuje się z następnym utworem
w punkcie MIX. W trybie **SINGLE** zatrzymuje się na końcu każdego utworu.
Kontynuuj w rozdziale [Odtwarzacze](players.md).
