# Rozwiązywanie problemów

## Brak dźwięku {#no-sound}

1. Otwórz **Ustawienia → Wyjścia audio** i naciśnij **Test Main** dla
   odtwarzacza. Jeśli słyszysz ton, sprawdź suwak głośności odtwarzacza.
2. Jeśli nic nie słyszysz, wybierz inne urządzenie lub parę kanałów. Zmiana
   wyjść zaczyna działać, gdy nic na nich nie gra; **Zastosuj teraz** w
   Ustawieniach stosuje ją od razu.
3. W Linuksie wybieraj **PipeWire** lub **PulseAudio** w Ustawienia →
   Wyjścia audio → System audio. Współdzielą kartę dźwiękową z innymi
   programami. **ALSA** rozmawia z kartą bezpośrednio i może ją zastać
   zajętą.
4. **JACK** jest pokazywany jako niedostępny („no output device”), gdy nie
   działa żaden serwer JACK. Uruchom serwer (na przykład przez QjackCtl) i
   uruchom aplikację ponownie. Ustaw serwer JACK na częstotliwość
   próbkowania z Ustawień (domyślnie 48 kHz): JACK pracuje z jedną
   częstotliwością dla każdego programu.
5. **PipeWire** nie jest oferowany przez archiwa do pobrania; docierają one
   do PipeWire przez jego usługę PulseAudio, która działa tak samo. Jest
   dostępny w wersjach zbudowanych z funkcją `pipewire`.

## Bit-perfect {#bit-perfect}

- **Wskaźnik BP pozostaje wyłączony.** Sprawdź każdy warunek w rozdziale
  [Wyjście bit-perfect](bit-perfect.md#the-bp-badge): brak wyciszania, nic
  innego na tych samych wyjściach, bezstratny plik, który został
  przeanalizowany, i urządzenie pracujące z częstotliwością pliku.
- **Krótka cisza przed utworem.** Urządzenie bit-perfect zostało otwarte
  ponownie z częstotliwością próbkowania utworu. Dzieje się tak tylko wtedy,
  gdy częstotliwość się zmienia: utwory z tą samą częstotliwością następują
  po sobie bez przerwy. Aby tego uniknąć, trzymaj bibliotekę przy jednej
  częstotliwości. Zob. [Jakie są koszty](bit-perfect.md#what-it-costs).
- **Utwory się nie miksują, a Play nie wycisza.** W odtwarzaczu, którego
  wyjście Main jest bit-perfect, każdy utwór gra do końca, po czym startuje
  następny, a Play / Następny i Poprzedni przełączają od razu: w takim
  odtwarzaczu nie ma przenikania ani segue. Wyłącz bit-perfect dla
  urządzenia albo przenieś odtwarzacz na inne wyjście, aby znów miksować. Zob.
  [Jakie są koszty](bit-perfect.md#what-it-costs).
- **Suwak głośności się nie rusza.** Wyjście Main odtwarzacza jest
  bit-perfect, co utrzymuje głośność na 100 %; mówi o tym podpowiedź suwaka.
  Nie poruszą go też suwaki MIDI ani API zdalne. Zob. [Jakie są
  koszty](bit-perfect.md#what-it-costs).
- **Utwór gra po resamplingu, a log mówi, że urządzenie jest zajęte
  (Linux).** Aby zmienić częstotliwość, aplikacja zamyka urządzenie i
  otwiera je ponownie. W tej chwili serwer dźwięku (PipeWire) może zabrać
  kartę. Aplikacja próbuje kilka razy; jeśli karta nadal jest zajęta, utwór
  gra z bieżącą częstotliwością urządzenia, a następny utwór znów prosi o
  swoją. Aby oddać aplikacji kartę na wyłączność, otwórz ustawienia dźwięku
  systemu i ustaw profil tej karty na **Off** (lub **Pro Audio**), aby serwer
  dźwięku zostawił jej urządzenie `hw:` w spokoju. Liczba prób i czas
  oczekiwania między nimi to `tuning.device_busy_retries` i
  `tuning.device_busy_retry_ms` w pliku konfiguracyjnym.
- **Urządzenie gra, ale wskaźnik BP pozostaje wyłączony (Windows lub
  macOS).** Dostęp na wyłączność został odrzucony, a urządzenie gra
  współdzielone.
  - Windows: inny program może trzymać urządzenie na wyłączność albo
    wyłączna kontrola jest wyłączona we właściwościach zaawansowanych
    urządzenia.
  - macOS: inny program może trzymać urządzenie w trybie hog albo urządzenie
    oferuje swoje częstotliwości tylko jako ciągły zakres (większość
    interfejsów wymienia stałe częstotliwości).
- **Urządzenia `hw:` nie można otworzyć (Linux).**
  - Kartę może trzymać serwer dźwięku. Zatrzymaj go albo ustaw, by zostawił
    tę kartę w spokoju, i uruchom aplikację ponownie.
  - Niektóre przetworniki USB przyjmują tylko upakowane próbki 24-bitowe
    (`S24_3LE`), których biblioteka audio nie obsługuje. Użyj tej karty przez
    `plughw:` (nie jest bit-perfect).

### DSD {#dsd}

- **Utwór DSD gra po konwersji, choć urządzenie jest ustawione na DoP lub
  natywne DSD.** Log mówi dlaczego („DSD converted to PCM” i przyczyna) w
  tych przypadkach: głośność odtwarzacza nie wynosi 100 %, coś innego gra na
  urządzeniu, więcej niż dwa kanały albo urządzenie odrzuca częstotliwość
  (DoP wymaga częstotliwości DSD podzielonej przez 16, na przykład 176,4 kHz
  dla DSD64) lub nie ma formatu 24- albo 32-bitowego. Utwór, który nie został
  jeszcze przeanalizowany, konwertuje się po cichu, bez wiersza w logu:
  przeanalizuj go (Ustawienia → Analiza) i odtwórz ponownie.
- **Tylko pierwszy utwór albumu DSD wychodzi jako DSD.** To domyślne
  ustawienie miksowania: utwory, które odtwarzacz uruchamia sam, grają
  skonwertowane. Wybierz **Zachowaj DSD i wycisz inne źródła** w Ustawienia →
  Wyjścia audio, aby zostały w DSD. Zob. [DSD](bit-perfect.md#dsd).
- **Nagłówek pokazuje DSD, ale przetwornik gra szum lub się nie
  synchronizuje.** Przetwornik nie rozpoznaje DoP (ani natywnego formatu).
  Ustaw urządzenie z powrotem na **Konwertuj na PCM**.
- **Trzask, gdy utwór DSD startuje, zatrzymuje się lub opuszcza DSD.**
  Przetwornik potrzebuje więcej ciszy DSD: zwiększ **Cisza DSD** (domyślnie
  200 ms) w Ustawienia → Wyjścia audio, Zaawansowane.
- **Inne odtwarzacze lub carty milczą na urządzeniu.** Gra utwór DSD z
  ustawieniem **Zachowaj DSD i wycisz inne źródła**; widać plakietkę **Inne
  wyciszone**. Znów zabrzmią, gdy utwór się skończy.

## Alert „Utracono wyjście” {#output-lost-alert}

Pasek stanu pokazuje **Utracono wyjście: &lt;device&gt;**, gdy urządzenie
przestaje odpowiadać. Odtwarzacze nadal liczą czas i miksują na wewnętrznym
zegarze, więc automatyka się nie zatrzymuje. Urządzenie jest ponawiane co
2 sekundy i przejmuje pracę z powrotem, gdy wróci. Podłącz ponownie kabel lub
włącz interfejs z powrotem.

### „Utracono wyjście”, które nigdy nie znika, przy bezpośrednim wyjściu `hw:` {#output-lost-that-never-clears-with-a-direct-hw-output}

Karta dźwiękowa używana przez bezpośrednie wyjście ALSA `hw:` (na przykład
wyjście bit-perfect) jest trzymana wyłącznie przez Fauste Player: serwer
dźwięku (PipeWire lub PulseAudio) nie może jej używać w tym samym czasie.
Jeśli inne wyjście idzie przez domyślne urządzenie serwera dźwięku, a to
domyślne urządzenie jest tą samą kartą, to wyjście nigdy nie startuje i
pozostaje w stanie **Utracono wyjście**. Log raz mówi „output device opened
but never started”.

Używaj jednej ścieżki na kartę: skieruj każde wyjście tej karty przez to samo
urządzenie `hw:` (w razie potrzeby z różnymi kanałami) albo wybierz inną
kartę jako domyślne wyjście serwera dźwięku w ustawieniach dźwięku systemu.

## Utwór pokazuje ikonę ostrzeżenia lub plik z krzyżykiem {#a-track-shows-a-warning-icon-or-a-file-with-a-cross}

Pliku brakuje (został przeniesiony, usunięty, odmontowany: plik z krzyżykiem)
albo nie można go zdekodować (znak ostrzeżenia). Odtwarzacze go pomijają.
Najedź na wiersz, aby zobaczyć, o który przypadek chodzi, oraz ścieżkę
pliku.

Brakującego pliku szuka się ponownie co 30 sekund
(`tuning.missing_recheck_ms` w pliku konfiguracyjnym): gdy dysk zostanie
zamontowany albo plik wróci na miejsce, utwór sam staje się odtwarzalny.
Plik, którego nie można zdekodować, jest sprawdzany ponownie samoczynnie na
tym samym zegarze, według rozmiaru i czasu modyfikacji: nie jest dekodowany
od nowa, chyba że któreś z nich się zmieniło, na przykład gdy kończy się
kopiowanie. Aby sprawdzić go od razu, użyj **Analizuj ponownie** w menu
jego wiersza, a dla całej biblioteki **Ustawienia → Analiza → Analizuj
ponownie wszystkie utwory**.

Plik WAV lub FLAC, który jest pokazywany jako nieczytelny albo zagrałby jako
głośny szum, może być zgraniem płyty DTS-CD: plik zawiera strumień bitowy DTS
zamiast dźwięku. Aplikacja go odrzuca (log mówi „encoded bitstream stored as
PCM”) i nie dekoduje DTS. Najpierw przekonwertuj go na zwykłe PCM innym
narzędziem.

## Obserwowany folder pokazuje ostrzeżenie {#a-watched-folder-shows-a-warning}

Kategoria obserwująca folder pokazuje ⟳, w innym kolorze, gdy folder nie jest
śledzony; najedź na nią, aby zobaczyć przyczynę. Zob. [Obserwowane
foldery](library.md#watched-folders).

- **Nie znaleziono lub nie można odczytać**: dysk albo udział nie jest
  zamontowany lub nie jest osiągalny. Pusty folder, który powinien zawierać
  utwory biblioteki, też liczy się jako nieznaleziony, bo niezamontowany udział
  często tak wygląda. Nic nie jest oznaczane ani usuwane; folderu szuka się
  ponownie co 30 sekund i skanuje się go, gdy się pojawi. Jeśli opróżniłeś
  folder celowo, wyczyść jego utwory, zaznaczając je w tabeli biblioteki i
  wybierając **Usuń z biblioteki…**, albo wybierając na nich **Analizuj
  ponownie**, co oznacza te, których plik zniknął, tak aby **Usuń brakujące
  utwory…** mogło je usunąć.
- **Zmiany nie są śledzone**: system nie chciał go obserwować, zwykle
  folder z bardzo wieloma podfolderami. Folder jest nadal skanowany przy
  każdym uruchomieniu i przy każdej ponownej próbie, więc nowe pliki pojawiają
  się wtedy, a nie od razu. W Linuksie drzewo folderów większe niż limit
  obserwacji na użytkownika wymaga wyższego `fs.inotify.max_user_watches` (na
  przykład `sudo sysctl fs.inotify.max_user_watches=524288`; ustaw to na
  stałe w `/etc/sysctl.d/`).
- **Nie pełna ścieżka**: zapisany folder został ręcznie zmieniony na ścieżkę,
  która nie jest pełna. Wybierz **Zmień obserwowany folder…**, aby wskazać go
  ponownie.
- **Udział sieciowy nie pokazuje ostrzeżenia, ale pliki dodane z innego
  komputera się nie pojawiają**: na udziale sieciowym (SMB/CIFS, NFS) system
  zwykle przyjmuje obserwację, ale zgłasza tylko zmiany wykonane z tego
  komputera. Zmiany wykonane gdzie indziej są widoczne przy następnym
  uruchomieniu, nie gdy aplikacja działa.
- **Folder z tysiącami plików** skanuje się przy starcie dłuższą chwilę;
  pokazuje to postęp analizy.
- Utwory, których plik opuścił obserwowany folder, zostają w bibliotece
  oznaczone jako brakujące. Usuń je przez **Usuń brakujące utwory…** w menu
  kategorii.

## Przerwy w dźwięku {#audio-dropouts}

Pasek stanu ostrzega przez 5 sekund po każdej przerwie wykrytej przez
aplikację: **P1: przerwy w dźwięku (3)**, gdy dekodowanie odtwarzacza nie
nadążyło za dyskiem (liczba dotyczy utworu, który teraz gra), oraz
**&lt;device&gt;: przerwy urządzenia audio (2)**, gdy urządzenie wyjściowe
nie dotrzymało terminu (xrun). Log też zapisuje każdą z nich, najwyżej jeden
wiersz co 10 sekund na rodzaj, z liczbą, ile ich było. Nie każdy system audio
zgłasza xruny (PulseAudio nie; tryb wyłączny Windows też nie).


- Zwiększ **rozmiar bufora** w Ustawieniach (działa, gdy nic nie gra na tym
  urządzeniu, albo od razu po **Zastosuj teraz**).
- Przez serwer dźwięku (PulseAudio, urządzenie ALSA `default`) każde wyjście
  już prosi o co najmniej 50 ms bez względu na rozmiar bufora, aby zajęty
  komputer nie był słyszalny; Ustawienia pokazują to obok opóźnienia. Na
  starym lub mocno obciążonym komputerze zwiększ `shared_buffer_ms` w sekcji
  `outputs` pliku konfiguracyjnego (na przykład do 100). System trzyma w
  kolejce około dwóch takich bloków, więc przyciski carta i play reagują
  wtedy około dwa razy tyle później. Dla wyjścia antenowego własny wpis
  sprzętowy urządzenia (`hw:` w Linuksie) całkiem omija serwer dźwięku.
- W Linuksie zezwól na planowanie w czasie rzeczywistym. Aplikacja prosi o
  nie system przez rtkit (D-Bus), a log zapisuje odmowę. Wiele pulpitów
  odmawia tego aplikacjom.
- Unikaj dysków sieciowych dla muzyki, która gra na antenie.

## Interfejs działa wolno na starym komputerze {#the-interface-is-slow-on-an-old-computer}

Gdy coś gra, interfejs rysuje najwyżej 30 klatek na sekundę (`max_fps` w
sekcji `ui` pliku konfiguracyjnego), a kliknięcia i klawisze są obsługiwane od
razu. Na starym lub zajętym komputerze zmniejsz tę wartość (na przykład do
20), aby zostawić więcej procesora dźwiękowi; 0 rysuje każdą klatkę, jaką
pokazuje wyświetlacz. Mierniki mierzą tak samo przy każdej częstotliwości;
są tylko rysowane rzadziej.

## Nie można upuszczać plików w Linuksie {#files-cannot-be-dropped-on-linux}

Okno przyjmuje pliki przeciągnięte z menedżera plików tylko wtedy, gdy działa
w X11. Domyślnie tak jest, także na pulpicie Wayland (przez XWayland, który
mają wszystkie pulpity). Jeśli `ui.window_system` w `config.json` ma wartość
`"Wayland"`, okno jest natywnym oknem Wayland, które może wyglądać ostrzej
przy ułamkowym skalowaniu wyświetlacza, ale nie można na nie upuszczać
plików: użyj zamiast tego **+ Dodaj**. Ustaw z powrotem `"Auto"` (domyślne)
lub `"X11"` i uruchom Fauste Player ponownie.

## Kontroler MIDI nie reaguje {#a-midi-controller-does-not-respond}

Jeśli **Ustawienia → MIDI** pokazuje kontroler jako podłączony, ale jego
przyciski i suwaki nic nie robią, naciśnij tam **Połącz porty ponownie**:
program zamyka porty MIDI i otwiera je od nowa, tak jak odłączenie i
podłączenie kontrolera. Jeśli to nie pomaga, odłącz kontroler i podłącz go
z powrotem.

W obu przypadkach log mówi, co się stało. Wymienia porty MIDI przy każdej ich
zmianie, każdy otwarty i zamknięty port (z przyczyną), port, którego nie da
się otworzyć (raz, dopóki się nie otworzy lub nie zniknie), pierwszy
komunikat otrzymany na każdym połączeniu, pierwszy komunikat zignorowany na
każdym połączeniu z przyczyną (brak przypisania, suwak, który nie dotarł
jeszcze do głośności, akcja, która nie może teraz działać) oraz pierwszy
komunikat diody, który się nie powiódł. Podłączony kontroler, który nigdy nie
zapisuje pierwszego komunikatu, nie wysyła niczego, co program otrzymuje. Aby
zobaczyć każdy zignorowany komunikat i każdą nieudaną ponowną próbę,
uruchom program z `RUST_LOG=info,fp_control=debug`.

## „W interfejsie wystąpił błąd” {#the-interface-hit-an-error}

Przechwycono błąd rysowania. Dźwięk nie jest dotknięty. Naciśnij **Uruchom
ponownie interfejs**. Zgłoś go, prosimy, razem z logami.

## Logi i raporty o awariach {#logs-and-crash-reports}

Folder z logami opisuje rozdział [Dane i kopie zapasowe](data-and-backups.md).
Jest jeden plik logu na dzień. Identyczne wiersze powtarzane jeden po drugim
są zapisywane raz, a po nich wiersz w rodzaju `previous message repeated 12
times` (podczas długiej serii powtórzeń ten wiersz jest zapisywany co minutę).
Wiersze, które się różnią, na przykład dlatego, że niosą licznik, nie są
zwijane. Rutynowe komunikaty połączenia klienta PulseAudio są domyślnie
pomijane. Ustaw `RUST_LOG=debug` w środowisku, aby uzyskać więcej szczegółów
(zastępuje to domyślny filtr `info,pulseaudio=warn`).

Log jest ograniczony: plik z jednego dnia przestaje rosnąć przy 10 MiB (jego
ostatni wiersz o tym mówi; reszta tego dnia nie jest zapisywana, do czego
prowadzi zalew różniących się wierszy), a zachowywanych jest ostatnich 14
plików. Zmienisz to w pliku konfiguracyjnym przez `limits.max_log_file_mb`
(od 1 do 1024), `limits.max_log_files` (od 2 do 365) i
`limits.log_repeat_summary_secs` (od 1 do 3600, domyślnie 60, jak często
zgłaszana jest długa seria powtórzeń); obowiązują od następnego uruchomienia.
Raporty o awariach są zapisywane jako `crash-<time>.txt` w tym samym folderze
i mają osobny limit (`limits.max_crash_reports` na uruchomienie).
Dołącz oba pliki, zgłaszając błąd.
