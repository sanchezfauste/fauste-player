# Ustawienia

Otwórz **Ustawienia** na górnym pasku. Zamknij je przyciskiem **Zamknij**
lub klawiszem `Esc`. Większość zmian działa od razu i jest zapisywana
automatycznie.

Okno ma jeden rozmiar (900 × 640, mniejszy na małym ekranie) niezależnie od
sekcji, a sekcja przewija się wewnątrz niego. Każda sekcja ustawia etykiety
w jednej kolumnie.

Sekcje Odtwarzacze, Mierniki, Analiza i Skróty klawiszowe mają w nagłówku
przycisk **Przywróć domyślne**. Prosi o potwierdzenie, a potem resetuje
tylko tę sekcję (Odtwarzacze zachowują liczbę odtwarzaczy i język; Skróty
nie mają innego przycisku resetowania). Wyjścia audio, Playlisty, Cartwall,
MIDI i Zdalne go nie mają.

## Wymagany restart {#restart-pending}

Niektóre zmiany zaczynają działać dopiero po ponownym uruchomieniu
aplikacji: system audio, częstotliwość próbkowania, rozmiar bufora (także
własny urządzenia), wyjścia Main i Cue (odtwarzaczy i cartwalla), urządzenia
bit-perfect i ustawienia DSD. Liczba odtwarzaczy do nich nie należy: działa
od razu.

Częstotliwość lub bufor nadane urządzeniu liczą się tylko wtedy, gdy zmieniają
to, z czym urządzenie jest otwierane: nadanie urządzeniu tej samej wartości
co globalna albo wyczyszczenie takiej wartości nie oczekuje na restart.

Limity i strojenie silnika również obowiązują od następnego uruchomienia,
ale edytuje się je w pliku konfiguracyjnym przy zamkniętej aplikacji (zob.
[Dane i kopie zapasowe](data-and-backups.md)), więc nigdy nie są pokazywane
jako oczekujące.

Gdy któraś z tych zmian czeka, stopka Ustawień mówi „Niektóre zmiany
zadziałają po ponownym uruchomieniu.” i oferuje **Uruchom ponownie teraz**, a
górny pasek pokazuje plakietkę **Wymagany restart**. Najedź na plakietkę, aby
zobaczyć, co czeka. Krótki komunikat (na przykład że ustawienie zostało
zapisane) może na chwilę zająć miejsce tekstu stopki; **Uruchom ponownie
teraz** zostaje. Oba robią to samo:

- Gdy nic nie jest na antenie, **Uruchom ponownie teraz** (lub plakietka)
  uruchamia ponownie od razu.
- Gdy coś jest na antenie, pojawia się okno wymieniające to, co brzmi, z
  przyciskami **Zatrzymaj i uruchom ponownie** oraz **Anuluj**.

Najpierw zapisywana jest sesja, a dźwięk i sterowanie MIDI się zatrzymują,
potem aplikacja startuje ponownie z tym samym folderem danych
(`FAUSTE_HOME`), i nic nie trafia potem na antenę samo. Jeśli aplikacja nie
może wystartować ponownie (we Flatpaku także wtedy, gdy nowa nie wystartuje
na czas), mówi o tym; uruchom ją z menu aplikacji.

## Wyjścia audio {#audio-outputs}

Zmiany w tej sekcji czekają na restart: zob.
[Wymagany restart](#restart-pending).

![Ustawienia, Wyjścia audio, widok Podstawowe: przełącznik, system audio, częstotliwość próbkowania, rozmiar bufora oraz wyjścia Main i Cue każdego odtwarzacza (tu system cichy)](../../images/guide/settings-outputs.png)

U góry **Pokaż** wybiera **Podstawowe** lub **Zaawansowane**. Podstawowe
pokazuje system audio, częstotliwość próbkowania, rozmiar bufora i wyjścia.
Zaawansowane dodaje dla każdego urządzenia używanego przez wyjście jego
własną częstotliwość i bufor, przełącznik bit-perfect i tryb DSD, a potem
ustawienia DSD. Przełączanie widoków tylko pokazuje lub ukrywa wiersze:
nic nie jest zmieniane ani resetowane. Gdy Podstawowe ukrywa ustawienie,
które jest w użyciu, mówi o tym jeden wiersz. Gdy żadne wyjście nie używa
już urządzenia, jego własna częstotliwość i bufor, przełącznik bit-perfect i
tryb DSD są zapominane przy następnym uruchomieniu aplikacji: jeśli potem
wyjście znów go użyje, zaczyna od wartości globalnych. Do tego czasu
ponowny wybór (na przykład po zamianie dwóch urządzeń) je zachowuje.

| Ustawienie | Znaczenie |
|---|---|
| System audio | Ostatni wybór, **Brak wyjścia (cisza)**, nie odtwarza niczego: osie czasu biegną w tempie rzeczywistym bez karty dźwiękowej (dla komputera bez niej lub do prób). Linux: PipeWire (w wersjach, które go zawierają), PulseAudio, JACK lub ALSA. Windows: WASAPI, ASIO (w wersjach, które go zawierają) lub JACK. macOS: Core Audio lub JACK. Systemy, których nie ma na tym komputerze, lub bez urządzenia wyjściowego (niedziałający serwer JACK), są pokazywane jako niedostępne. „Domyślny systemowy” używa pierwszego dostępnego w tej kolejności. |
| Częstotliwość próbkowania | Częstotliwość, z jaką działa każde wyjście, chyba że urządzenie ma własną (Zaawansowane); pliki są do niej konwertowane resamplingiem wysokiej jakości. Urządzenia bit-perfect startują z własną częstotliwością, a potem podążają za plikami. |
| Rozmiar bufora | Ramki na blok audio, chyba że urządzenie ma własny; wynikające z niego opóźnienie jest pokazane pod spodem |
| Wyjścia odtwarzaczy | Dla każdego odtwarzacza urządzenie **Main** (na antenę) i urządzenie **Cue** (odsłuch wstępny), każde z parą kanałów. Karta dźwiękowa oferująca kilka profili wyjściowych (ALSA wymienia front, surround, bezpośredni sprzęt…) pokazuje każdy jako *karta — profil*; dwie pozycje, które nadal brzmiałyby tak samo, dostają w nawiasie identyfikator urządzenia. Interfejsy wielokanałowe mogą obsługiwać kilka odtwarzaczy na różnych parach. |
| Test Main / Test Cue | Odtwarza krótki ton (1 kHz na Main, 440 Hz na Cue, 1,5 s, −18 dBFS) na wybranym wyjściu, abyś mógł sprawdzić okablowanie przed wejściem na antenę |
| Cartwall | Wyjścia Main i Cue cartwalla. Main domyślnie jest wyjściem systemowym. Bez Cue nie ma odsłuchu wstępnego cartów. |
| Częstotliwość próbkowania: *urządzenie* (Zaawansowane) | **Globalna (...)** używa częstotliwości powyżej; wartość nadaje temu urządzeniu własną. Oferowane są tylko częstotliwości zgłaszane przez urządzenie; zapisana częstotliwość, której już nie zgłasza, pozostaje na liście z uwagą, że może się nie otworzyć (urządzenie wraca wtedy do częstotliwości globalnej). Własne wartości dotyczą tylko urządzeń wskazanych przez wyjście, nie domyślnego wyjścia systemowego, chyba że któreś je wskazuje. |
| Rozmiar bufora: *urządzenie* (Zaawansowane) | **Globalny (...)** używa rozmiaru bufora powyżej; wartość nadaje temu urządzeniu własny, z opóźnieniem pod spodem. Urządzenie, które nie przyjmuje własnego rozmiaru bufora, wraca do globalnego, a do globalnej częstotliwości także wtedy, gdy nie przyjmuje własnej. |
| Bit-perfect: *urządzenie* (Zaawansowane) | Urządzenie bit-perfect jest otwierane z dostępem na wyłączność i podąża za częstotliwością próbkowania każdego pliku, gdy nic na nim nie gra. Przełącznik jest wyłączony tam, gdzie urządzenie nie może dać dostępu na wyłączność. Zob. [Wyjście bit-perfect](bit-perfect.md). |
| DSD: *urządzenie* (Zaawansowane) | **Konwertuj na PCM** (domyślnie), **DoP** lub, w Linuksie, **Natywne DSD**. Pokazuje je każde urządzenie; oferowane są tylko tryby, które urządzenie może przyjąć, a wiersz pod spodem mówi, dlaczego inne nie. Zob. [DSD](bit-perfect.md#dsd). |
| Gdy inne źródło potrzebuje wyjścia DSD (Zaawansowane) | **Kontynuuj utwór DSD jako PCM** (domyślnie) lub **Zachowaj DSD i wycisz inne źródła**. Zob. [DSD](bit-perfect.md#dsd). |
| Cisza DSD (Zaawansowane) | Cisza wysyłana przed rozpoczęciem strumienia DSD, po jego zakończeniu i przy przejściu na PCM, aby przetwornik zsynchronizował się bez trzasku; domyślnie 200 ms, od 0 do 2000. |

Wyjście Cue nigdy nie wraca do wyjścia, którego używa Main, aby odsłuch
wstępny nigdy nie trafił na antenę. Cue wskazujące urządzenie w systemie
audio, którego ten komputer nie ma, lub to samo wyjście (urządzenie i kanały)
co Main oznacza „brak cue”. Gdy wyjście Cue jest takie samo jak jego wyjście
Main, ostrzeżenie pod nim o tym informuje. Odtwarzacz bez wyjścia Cue albo z
Cue na wyjściu Main ma przyciemniony przycisk **CUE**; po najechaniu kursorem
podpowiada, aby wybrać tutaj wyjście Cue. To samo dotyczy **Odsłuchaj na
CUE** w cartwallu.

Jeśli urządzenie zniknie podczas odtwarzania, odtwarzacze zachowują swoje
osie czasu, a urządzenie jest otwierane ponownie, gdy wróci (zob.
[Rozwiązywanie problemów](troubleshooting.md)).

## Odtwarzacze {#players}

![Ustawienia, Odtwarzacze: liczba odtwarzaczy, tryb domyślny, czas wyciszania, automatyczny miks, cue-in i cue-out, ostrzeżenie o końcu utworu i język](../../images/guide/settings-players.png)

| Ustawienie | Domyślnie | Znaczenie |
|---|---|---|
| Język | Systemowy | Język interfejsu |
| Liczba odtwarzaczy | 4 | Kolumny na ekranie głównym (nie można usunąć odtwarzacza na antenie) |
| Tryb domyślny | CONT | Tryb, w którym startują odtwarzacze |
| Czas wyciszania | 1000 ms | Używany przez Play na antenie i przez Stop z wyciszeniem |
| Automatyczny miks w punkcie MIX | Wł. | Nakłada utwory w trybie ciągłym |
| Używaj cue-in i cue-out | Wł. | Wył.: odtwarzacze grają każdy utwór od początku do końca pliku; markery są zachowywane, a carty nadal używają swoich. Czasy trwania i sumy playlist podążają za tym samym zakresem |
| Ostrzeżenie o końcu utworu | 10 s | Kiedy odliczanie zaczyna migać na czerwono |

## Mierniki {#meters}

![Ustawienia, Mierniki, z wybranym miernikiem szczytu cyfrowego](../../images/guide/settings-meters.png)

Zmiany działają od razu. Ustawienia pokazują tylko to, czego używa wybrany
typ miernika: mierniki EBU, DIN i VU mają skalę, strefę czerwoną i zachowanie
ustalone przez swoją normę (ustawia się tylko poziom odniesienia), a
odniesienie miernika K-System jest jego własnym 0. Ustawiona przez Ciebie
wartość jest zachowywana na czas, gdy ponownie wybierzesz ten typ.

| Ustawienie | Domyślnie | Znaczenie |
|---|---|---|
| Typ miernika | Szczyt cyfrowy | Jak słupek rośnie i opada oraz jaka jest jego skala, zgodnie z normą (zob. niżej) |
| Czas narastania, Szybkość opadania | 5 ms, 11,8 dB/s | Tylko dla **Własny**. Czas narastania to czas całkowania: impuls tonu tak długi odczytuje się o 2 dB za nisko; 0 pokazuje każdy szczyt. |
| True peak | Wył. | Tylko szczyt cyfrowy, własny i K-System. Mierzy między próbkami, z filtrem nadpróbkowania 4×, który publikuje ITU-R BS.1770. Pokazuje szczyty przekraczające 0 dBFS po konwersji, których miernik szczytu próbek nie widzi. Jak dopuszcza norma, pojedynczy trzask jednej próbki może odczytać się nawet o około 0,3 dB poniżej wartości swojej próbki. |
| Dół skali | −60 dBFS | Dół skali cyfrowej (szczyt cyfrowy i własny). Pozostałe mierniki pokazują zakres wynikający z ich normy. |
| Podtrzymanie szczytu | 2 s | Tylko szczyt cyfrowy, własny i K-System: jak długo najwyższy poziom pozostaje podświetlony; 0 wyłącza. Mierniki programowe i VU nie mają podtrzymania. |
| Poziom odniesienia | −18 dBFS | Wszystkie oprócz K-System. Zaznaczony na skali (EBU R68). Tu też leżą znacznik TEST EBU, znacznik −9 DIN i 0 VU. |
| Ostrzeżenie od | −9 dBFS | Żółty od tego poziomu (maksimum dopuszczalne wg EBU), dla mierników szczytu cyfrowego i własnego |
| Zagrożenie od | −3 dBFS | Czerwony od tego poziomu, dla mierników szczytu cyfrowego i własnego. Pozostałe czerwienieją tam, gdzie ich skala: VU od 0 VU, PPM EBU i DIN od dozwolonego maksimum (EBU +9, DIN 0), K-System od +4. |
| Odczyt głośności | Krótkookresowa | Głośność pod miernikiem: wył., chwilowa (ostatnie 400 ms) lub krótkookresowa (ostatnie 3 s), EBU R128 |
| Docelowa głośność | −23 LUFS | Odczyt jest zielony w granicach ±1 LU (EBU R128) |

| Typ miernika | Norma | Zachowanie |
|---|---|---|
| Szczyt cyfrowy | IEC 60268-18 | Pokazuje każdy szczyt od razu; opada o 20 dB w 1,7 s |
| PPM EBU | IEC 60268-10 typ IIb | Szczyty krótsze niż około 10 ms odczytują się niżej (impuls tonu 10 ms odczytuje się o około 1,6 dB za nisko, a 0,5 ms o około 18 dB), w granicach tolerancji EBU Tech 3205; opada o 24 dB w 2,8 s |
| PPM DIN | IEC 60268-10 typ I | To samo z czasem całkowania 5 ms; opada o 20 dB w 1,5 s |
| VU | IEC 60268-17 | Poziom średni, z ruchem wskazówki miernika VU: 99 % w 300 ms, z lekkim przeregulowaniem; sinus odczytuje się na swoim poziomie szczytowym |
| K-20, K-14, K-12 | K-System | Dwie części: poziom średni (RMS, 600 ms) jako pełny słupek i szczyt (opada o 26 dB w 3 s) przyciemniony nad nim. 0 leży 20, 14 lub 12 dB poniżej pełnej skali; zielony poniżej 0, bursztynowy od 0 do +4, czerwony powyżej. K-12 pasuje do radiofonii, K-14 i K-20 do programu o większej dynamice. |
| Własny | — | Twój czas narastania i szybkość opadania |

Każdy miernik używa skali swojej normy, ze znacznikami między kanałami:

| Miernik | Skala |
|---|---|
| Szczyt cyfrowy, własny | −60 … 0 dBFS, znaczniki co 10 dB do −40 i co 5 dB powyżej; górne 20 dB zajmuje połowę wysokości |
| PPM EBU | −12 … +12 wokół poziomu odniesienia (TEST), co 4 dB; cichsze poziomy spoczywają na dole |
| PPM DIN | −50 … +5, gdzie 0 leży 9 dB powyżej poziomu odniesienia (domyślnie −9 dBFS) |
| VU | −20 … +3 VU, 0 VU na poziomie odniesienia; słupek porusza się proporcjonalnie do napięcia, jak wskazówka |
| K-System | od +20, +14 lub +12 (0 dBFS) w dół do −60; równo w dB do −24 |

## Analiza {#analysis}

![Ustawienia, Analiza: progi markerów automatycznych](../../images/guide/settings-analysis.png)

Progi opisane w rozdziale [Markery i miksowanie](markers-and-mixing.md).
**Analizuj ponownie wszystkie utwory** uruchamia analizę od nowa dla całej
biblioteki; markery ręczne są zachowywane.

Po aktualizacji, w której zmieniła się analiza, utwory przeanalizowane przez
wcześniejszą wersję zachowują swoje markery i przebiegi, które nadal
działają. Przy starcie Fauste Player mówi, ile ich jest, i oferuje
**Analizuj teraz** lub **Później**; **Analizuj nieaktualne utwory (N)** tutaj
robi to samo w dowolnej chwili. Utwory w odtwarzaczach i tak są aktualizowane
w miarę wyświetlania, podobnie jak utwory cartów, które nie mają zapisanego
formatu (cart gra bit-perfect tylko wtedy, gdy jego format jest znany).
Utwory, których pliku brakuje, nie są liczone, dopóki plik nie wróci.

## Playlisty {#playlists}

![Ustawienia, Playlisty: folder z muzyką, playlisty i kolumny tabeli](../../images/guide/settings-playlists.png)

- **Folder z muzyką:** od którego zaczynają się okna wyboru plików.
- **Nowa playlista**, **zmiana nazwy** (edytuj nazwę i naciśnij Enter; Esc
  anuluje) i **usuwanie** (ikona kosza).
- **Importuj M3U / PLS…** tworzy nową playlistę z pliku playlisty. **M3U** w
  każdym wierszu eksportuje ją jako M3U8. Zob. [Playlisty](playlists.md).
- **Kolumny tabeli:** które kolumny pokazują tabele utworów i w jakiej
  kolejności, dla każdego odtwarzacza: pole wyboru na kolumnę (Tytułu i
  Czasu nie można wyłączyć), strzałki w górę i w dół dla pokazanych oraz
  **Domyślne kolumny**. Zob. [Playlisty](playlists.md).

**Język:** lista rozwijana: **Systemowy** (podążaj za systemem operacyjnym),
potem każdy język, w którym dostępny jest interfejs, pod własną nazwą
(najpierw angielski, potem alfabetycznie: na przykład Español). Interfejs
przełącza się od razu. Język systemowy bez własnego tłumaczenia używa
najbliższego (kanadyjski francuski używa francuskiego, brazylijski
portugalski używa portugalskiego), a w pozostałych przypadkach angielskiego.
Język w pliku ustawień, którego interfejs nie ma, jest pokazywany jako
**Systemowy** i podąża za systemem operacyjnym.

Angielska i hiszpańska wersja zostały napisane ręcznie. Pozostałe
tłumaczenia wygenerowano za pomocą AI i mogą zawierać błędy; gdy któreś z
nich jest w użyciu, **O programie** o tym informuje. Poprawki od rodzimych
użytkowników języka są mile widziane jako zgłoszenia (issues) lub pull
requesty.

## Cartwall {#cartwall}

![Ustawienia, Cartwall: strony, siatka i edytor wybranego carta](../../images/guide/settings-cartwall.png)

Strony, rozmiar siatki, edytor cartów oraz import i eksport stron cartów.
Zob. [Cartwall](cartwall.md).

## Skróty klawiszowe {#keyboard-shortcuts}

![Ustawienia, Skróty klawiszowe: każda akcja odtwarzacza z jej klawiszem i przycisk Odepnij obok przypisanych](../../images/guide/settings-shortcuts.png)

Zob. [Klawiatura](keyboard.md).

## MIDI {#midi}

![Ustawienia, MIDI, ze sterowaniem MIDI wyłączonym](../../images/guide/settings-midi.png)

Włącz powierzchnie sterujące MIDI, zobacz porty wejściowe i naucz kontrolkę
dla każdej akcji odtwarzacza. Zob. [Powierzchnie sterujące MIDI](midi.md).

## Zdalne {#remote}

![Ustawienia, Zdalne, z API HTTP nasłuchującym na tym komputerze](../../images/guide/settings-remote.png)

Zdalne sterowanie przez sieć, dla stron WWW, aplikacji na telefon,
automatyki i powierzchni sterujących. Zob. [Zdalne sterowanie](remote-control.md).

- **Zezwalaj na sterowanie zdalne przez HTTP**, jego **adres** i **port**
  oraz wiersz mówiący, czy nasłuchuje.
- **Token**, wymagany poza tym komputerem. **Generuj** tworzy losowy,
  **Pokaż** go odsłania, a **Kopiuj** umieszcza go w schowku. Ostrzeżenie
  pojawia się, gdy adres sięga poza ten komputer, a tokenu nie ma.
- **Strony WWW, które mogą używać API**: jeden origin w wierszu.
- **Zezwalaj na sterowanie przez OSC**, jego **adres** i **port** oraz
  **dozwoleni nadawcy** (adresy lub podsieci, jeden w wierszu).
- **Publikuj czasy co**: jak często wysyłane są czasy, które upłynęły i
  które pozostały, gdy coś gra.

Pola tekstowe i liczby są stosowane, gdy je opuścisz, co obejmuje otwarcie
innej sekcji lub zamknięcie Ustawień; Esc anuluje to, co wpisywałeś.
Nieprawidłowa wartość jest poprawiana, a pole pokazuje, co zostało
zachowane.
