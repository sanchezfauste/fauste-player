# Pierwsze kroki

## Instalacja {#install}

W Windows Fauste Player jest w **Microsoft Store**
(<https://apps.microsoft.com/detail/9NH4KCL73463>), a w Linuksie w
**Snap Store** (<https://snapcraft.io/fauste-player>). Możesz też pobrać pakiet
dla swojego systemu ze strony **Releases** projektu. Obok
każdego pliku leży plik `.sha256`. Aby sprawdzić pobrany plik, uruchom
`sha256sum -c <file>.sha256` w Linuksie lub `shasum -a 256 -c <file>.sha256`
w macOS.

### Linux {#linux}

| Pakiet | Instalacja | Aktualizacja | Usunięcie |
|---|---|---|---|
| Snap (Snap Store) | `sudo snap install fauste-player` | automatyczna (`sudo snap refresh fauste-player`, aby zaktualizować teraz) | `sudo snap remove fauste-player` |
| `.deb` (Debian, Ubuntu) | `sudo apt install ./fauste-player_<version>_amd64.deb` | zainstaluj nowszy `.deb` w ten sam sposób | `sudo apt remove fauste-player` |
| `.rpm` (Fedora, openSUSE) | `sudo dnf install ./fauste-player-<version>-1.x86_64.rpm` | zainstaluj nowszy `.rpm` | `sudo dnf remove fauste-player` |
| AppImage (dowolna dystrybucja) | `chmod +x fauste-player-<version>-x86_64.AppImage`, a następnie uruchom plik | zastąp plik | usuń plik |
| Pakiet Flatpak | `flatpak install --user fauste-player-<version>-x86_64.flatpak` | zainstaluj nowszy pakiet | `flatpak uninstall com.sanchezfauste.FaustePlayer` |
| Archiwum | rozpakuj je i uruchom `fauste-player` | rozpakuj nowsze | usuń folder |

- **Integracja z pulpitem:** pakiety dodają **Fauste Player** do menu
  aplikacji i pozwalają otwierać nim playlisty `.m3u`, `.m3u8` i `.pls`,
  które są wtedy importowane jako nowe playlisty.
- **Wymagane biblioteki:** biblioteki ALSA i D-Bus. Mają je wszystkie
  środowiska graficzne, a pakiety je deklarują. JACK jest używany, gdy jest
  zainstalowany, i nigdy nie jest wymagany.
- **AppImage:** jeśli nie uruchamia się z powodu braku FUSE, uruchom go z
  opcją `--appimage-extract-and-run`.
- **Snap:** snap działa w piaskownicy. Od razu czyta Twój folder domowy (z
  wyjątkiem ukrytych plików i folderów), gra przez PulseAudio lub PipeWire i
  może używać sieci na potrzeby API zdalnego sterowania i OSC (oba wyłączone,
  dopóki ich nie włączysz). Dwie funkcje wymagają uprawnienia, które nadajesz
  raz, z terminala:
  - Wyjście bit-perfect na urządzeniach ALSA `hw:` oraz powierzchnie
    sterujące MIDI: `sudo snap connect fauste-player:alsa`.
  - Muzyka na dyskach USB i innych dyskach (`/media`, `/run/media`, `/mnt`):
    `sudo snap connect fauste-player:removable-media`.

  `snap connections fauste-player` pokazuje, co jest podłączone. Po
  podłączeniu uruchom Fauste Player ponownie. Muzyka w innych miejscach (poza
  folderem domowym i tymi trzema) nie może być odczytana przez snap: użyj do
  niej innego pakietu. JACK nie jest dostępny w snapie. Jego ustawienia i
  playlisty są trzymane w `~/snap/fauste-player/` (zob. [Dane i kopie
  zapasowe](data-and-backups.md)).
- **Flatpak:**
  - Pakiet wymaga środowiska uruchomieniowego freedesktop z Flathub. Jeśli
    zdalne repozytorium Flathub nie jest skonfigurowane, najpierw uruchom
    `flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo`.
  - Piaskownica może czytać Twój folder domowy (aby odtwarzać muzykę tam,
    gdzie leży).
  - Gra przez PulseAudio, a w przypadku urządzeń bit-perfect bezpośrednio
    przez ALSA.
  - Może używać sieci na potrzeby API zdalnego sterowania i OSC (oba
    wyłączone, dopóki ich nie włączysz).
  - Pakiety do wersji 0.10 były instalowane jako `org.fauste.FaustePlayer`, a
    nowsze instalują się jako `com.sanchezfauste.FaustePlayer`, osobna
    aplikacja. Aby zachować ustawienia, playlisty i carty, zamknij
    aplikację i uruchom
    `mv ~/.var/app/org.fauste.FaustePlayer ~/.var/app/com.sanchezfauste.FaustePlayer`
    przed uruchomieniem nowej, a potem
    `flatpak uninstall org.fauste.FaustePlayer`.
- **Biblioteki pulpitu:** okno korzysta z libxkbcommon oraz EGL lub OpenGL,
  z Waylandem lub X11. Mają je wszystkie środowiska graficzne, a pakiety
  `.deb` i `.rpm` je deklarują. W bardzo minimalnym systemie zainstaluj je
  przed użyciem AppImage.

### Windows {#windows}

Najłatwiej przez **Microsoft Store**: wyszukaj *Fauste Player* albo otwórz
<https://apps.microsoft.com/detail/9NH4KCL73463> i wybierz **Pobierz**. Sklep
dba o aktualność aplikacji, a **Ustawienia → Aplikacje** ją usuwa.

Albo uruchom `fauste-player-<version>-x86_64-pc-windows-msvc.msi`. Instaluje się
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
**Ustawień**) otwiera okno **O programie** z autorem, prawami autorskimi,
licencją (GNU GPL w wersji 3 lub nowszej; **Przeczytaj licencję online**
otwiera jej tekst), krótkim zastrzeżeniem i informacjami o licencjach
(**Licencje stron trzecich** otwiera plik z informacjami instalowany z
pakietami wydań). Zamkniesz je przyciskiem **Zamknij** lub klawiszem `Esc`.
`fauste-player --help` wyświetla listę opcji. Pliki playlist podane jako argumenty są importowane jako nowe
playlisty.

![Okno O programie: wersja, prawa autorskie i licencje dołączonych komponentów](../../images/guide/about.png)

W Windows program wydania nie otwiera okna konsoli: `--version`, `--help` i
błędy uruchamiania pojawiają się zamiast tego w oknie komunikatu.

Gdy zmiana wyjść czeka na coś, co jest na antenie, na górnym pasku pojawia
się plakietka **Oczekujące ustawienia**; mówi ona, na co się czeka (zob.
[Ustawienia](settings.md#settings-pending)).

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
pliki DFF skompresowane DST. Plik WAV lub FLAC zawierający strumień bitowy
DTS (lub inny zakodowany dźwięk przestrzenny) zamiast dźwięku, jak w
niektórych zgraniach płyt DTS-CD, też jest pokazywany jako nieczytelny (log
mówi dlaczego): najpierw przekonwertuj go na zwykłe PCM.

Pliki przestrzenne (quad, 5.1, 6.1, 7.1 i inne układy do ośmiu kanałów) grają
zmiksowane do stereo według położenia każdego kanału: przedni lewy i prawy
zostają po swojej stronie, środkowy i tylny środkowy trafiają na obie strony,
kanały boczne i tylne trafiają na swoją stronę, a kanał LFE (subwoofer) jest
pomijany. Wielokanałowe pliki Opus i AAC nie mogą być odczytane i są
pokazywane jako nieczytelne.

Każdy plik jest analizowany w tle. Analiza odczytuje tytuł, wykonawcę, album
i okładkę, rysuje przebieg oraz wyznacza, gdzie dźwięk się zaczyna i kończy
oraz gdzie wejść miksem w następny utwór. Utwór możesz odtworzyć, zanim jego
analiza się zakończy. Gdy pliki są analizowane, prawa strona paska stanu
mówi, ile ich jest (**Analiza 12 utworów**); najedź na nią, aby zobaczyć
pliki analizowane teraz, ile czeka i, podczas **Analizuj ponownie wszystkie
utwory**, jak daleko to zaszło. Legenda kolorów jest na prawo od niej; gdy okno
jest wąskie, najpierw ukrywane są pozycje legendy, potem ta pozycja, aby
ostrzeżenie po lewej stronie paska stanu pozostało czytelne. Gdy działają
tylko pomiary głośności (zob. [Głośność](loudness.md)), pasek mówi **Pomiar
głośności 5 utworów**.

## Odtwarzanie {#play}

- Naciśnij **Play** (albo klawisz numeryczny odtwarzacza, `1` dla P1), aby
  uruchomić **następny** utwór, oznaczony na liście na zielono.
- Naciśnij **Play** ponownie, gdy utwór jest na antenie, aby przejść
  płynnie do następnego.
- **Kliknij dwukrotnie** utwór, aby ustawić go jako następny.

W trybie **CONT** (ciągłym) odtwarzacz sam miksuje się z następnym utworem
w punkcie MIX. W trybie **SINGLE** zatrzymuje się na końcu każdego utworu.
Kontynuuj w rozdziale [Odtwarzacze](players.md).
