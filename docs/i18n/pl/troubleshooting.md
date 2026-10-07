# Rozwiązywanie problemów

## Brak dźwięku {#no-sound}

1. Otwórz **Ustawienia → Wyjścia audio** i naciśnij **Test Main** dla
   odtwarzacza. Jeśli słyszysz ton, sprawdź suwak głośności odtwarzacza.
2. Jeśli nic nie słyszysz, wybierz inne urządzenie lub parę kanałów. Zmiany
   wyjść zaczynają działać po restarcie: naciśnij **Uruchom ponownie teraz**
   w Ustawieniach.
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
  [Wyjście bit-perfect](bit-perfect.md#the-bp-badge): głośność na 100 %, brak
  wyciszania, nic innego na tych samych wyjściach, bezstratny plik, który
  został przeanalizowany, i urządzenie pracujące z częstotliwością pliku.
- **Krótka cisza przed utworem.** Urządzenie bit-perfect zostało otwarte
  ponownie z częstotliwością próbkowania utworu. Aby tego uniknąć, trzymaj
  bibliotekę przy jednej częstotliwości.
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

## Przerwy w dźwięku {#audio-dropouts}

Pasek stanu ostrzega przez 5 sekund po każdej przerwie wykrytej przez
aplikację: **P1: przerwy w dźwięku (3)**, gdy dekodowanie odtwarzacza nie
nadążyło za dyskiem (liczba dotyczy utworu, który teraz gra), oraz
**&lt;device&gt;: przerwy urządzenia audio (2)**, gdy urządzenie wyjściowe
nie dotrzymało terminu (xrun). Log też zapisuje każdą z nich, najwyżej jeden
wiersz co 10 sekund na rodzaj, z liczbą, ile ich było. Nie każdy system audio
zgłasza xruny (PulseAudio nie; tryb wyłączny Windows też nie).


- Zwiększ **rozmiar bufora** w Ustawieniach (i naciśnij **Uruchom ponownie
  teraz**).
- W Linuksie zezwól na planowanie w czasie rzeczywistym. Aplikacja prosi o
  nie system przez rtkit (D-Bus). Działa też członkostwo w grupie `audio` z
  limitem `rtprio`.
- Unikaj dysków sieciowych dla muzyki, która gra na antenie.

## „W interfejsie wystąpił błąd” {#the-interface-hit-an-error}

Przechwycono błąd rysowania. Dźwięk nie jest dotknięty. Naciśnij **Uruchom
ponownie interfejs**. Zgłoś go, prosimy, razem z logami.

## Logi i raporty o awariach {#logs-and-crash-reports}

Folder z logami opisuje rozdział [Dane i kopie zapasowe](data-and-backups.md).
Jest jeden plik logu na dzień, a zachowywanych jest ostatnich 14. Raporty o
awariach są zapisywane jako `crash-<time>.txt`. Ustaw `RUST_LOG=debug` w
środowisku, aby uzyskać więcej szczegółów. Dołącz oba pliki, zgłaszając błąd.
