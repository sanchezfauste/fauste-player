# Ustawienia

Otwórz **Ustawienia** na górnym pasku. Zamknij je przyciskiem **Zamknij**
lub klawiszem `Esc`. Większość zmian działa od razu i jest zapisywana
automatycznie.

Okno ma jeden rozmiar (900 × 640, mniejszy na małym ekranie) niezależnie od
sekcji, a sekcja przewija się wewnątrz niego. Każda sekcja ustawia etykiety
w jednej kolumnie.

Sekcje Odtwarzacze, Mierniki, Analiza, Głośność i Skróty klawiszowe mają w nagłówku
przycisk **Przywróć domyślne**. Prosi o potwierdzenie, a potem resetuje
tylko tę sekcję (Odtwarzacze zachowują liczbę odtwarzaczy i język; Skróty
nie mają innego przycisku resetowania). Wyjścia audio, Playlisty, Cartwall,
Biblioteka, MIDI i Zdalne go nie mają.

## Oczekujące ustawienia {#settings-pending}

Każde ustawienie obowiązuje, gdy aplikacja działa. System audio,
częstotliwość próbkowania, rozmiar bufora (także własny urządzenia), wyjścia
Main i Cue (odtwarzaczy i cartwalla), urządzenia bit-perfect i ustawienia DSD
zmieniają sposób otwierania urządzenia wyjściowego, więc obowiązują, gdy tylko
jest to bezpieczne:

- Ustawienia urządzenia obowiązują, gdy nic nie gra na tym urządzeniu: żaden
  skierowany na nie odtwarzacz nie gra, nie wycisza się ani nie odsłuchuje
  wstępnie (CUE, także przytrzymane), i żaden cart nie gra na nim. Wstrzymany
  odtwarzacz albo taki, w którym utwór tylko jest wczytany, nie trzyma
  urządzenia: jego utwór zostaje wstrzymany tam, gdzie był.
- Wyjście odtwarzacza lub cartwalla przenosi się, gdy ten odtwarzacz (lub
  cartwall) nie gra; inne odtwarzacze na tych samych urządzeniach nie są
  przerywane.
- Nowy system audio obowiązuje, gdy nigdzie nic nie gra.

Oczekująca zmiana jest wykonywana w pierwszej chwili, gdy nic, czego dotyczy,
nie gra. Może to być przerwa między końcem jednego utworu a startem
następnego: następny utwór startuje wtedy później o czas, którego urządzenie
potrzebuje na ponowne otwarcie. **Zastosuj teraz** (niżej) wymusza zmianę od
razu.

Częstotliwość lub bufor nadane urządzeniu liczą się tylko wtedy, gdy zmieniają
to, z czym urządzenie jest otwierane: nadanie urządzeniu tej samej wartości
co globalna albo wyczyszczenie takiej wartości niczego nie zmienia.

Do tego czasu górny pasek pokazuje **Oczekujące ustawienia**. Najedź na nią,
aby zobaczyć, co czeka i na co; kliknij ją, aby zobaczyć listę, która
pokazuje też każdą zmianę odrzuconą przez urządzenie lub wyjście (urządzenie
zachowuje bieżące ustawienia; zmiana jest podejmowana ponownie, gdy ją
zmienisz lub naciśniesz **Zastosuj teraz**). Stopka Ustawień mówi „Niektóre
zmiany czekają, aż wyjścia, których dotyczą, będą wolne.” i oferuje
**Zastosuj teraz**; krótki komunikat (na przykład że ustawienie zostało
zapisane) może na chwilę zająć miejsce tego tekstu.

**Zastosuj teraz** stosuje od razu każdą oczekującą zmianę wyjść. Gdy
cokolwiek by to przerwało, najpierw wymienia urządzenia i to, co na nich gra;
**Przerwij i zastosuj** na krótko przerywa wtedy dźwięk na tych urządzeniach:
to, co grało, idzie dalej od miejsca, w którym było, po krótkiej przerwie, a
nic wstrzymanego ani zatrzymanego nie startuje. **Anuluj** (lub `Esc`) dalej
czeka.

Limity i strojenie silnika edytuje się w pliku konfiguracyjnym przy
zamkniętej aplikacji (zob. [Dane i kopie zapasowe](data-and-backups.md)).

## Wyjścia audio {#audio-outputs}

Zmiany w tej sekcji obowiązują, gdy aplikacja działa, gdy tylko nic, czego
dotyczą, nie gra: zob. [Oczekujące ustawienia](#settings-pending).

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
| Rozmiar bufora | Ramki na blok audio, chyba że urządzenie ma własny; wynikające z niego opóźnienie jest pokazane pod spodem. Wyjścia przez serwer dźwięku lub mikser systemowy (PulseAudio, urządzenie ALSA `default`) proszą o co najmniej 50 ms bez względu na ten rozmiar, albo tyle, ile przyjmie urządzenie, aby zajęty komputer nie był słyszalny; opóźnienie mówi wtedy o tym (`shared_buffer_ms` w pliku konfiguracyjnym) |
| Wyjścia odtwarzaczy | Dla każdego odtwarzacza urządzenie **Main** (na antenę) i urządzenie **Cue** (odsłuch wstępny), każde z parą kanałów. Karta dźwiękowa oferująca kilka profili wyjściowych (ALSA wymienia front, surround, bezpośredni sprzęt…) pokazuje każdy jako *karta — profil*; dwie pozycje, które nadal brzmiałyby tak samo, dostają w nawiasie identyfikator urządzenia. Interfejsy wielokanałowe mogą obsługiwać kilka odtwarzaczy na różnych parach. Wyjście Cue dzielące wyjście antenowe (Main dowolnego odtwarzacza lub cartwalla) jest dozwolone: uwaga pod nim mówi, że jego odsłuch wstępny jest słyszalny na antenie, i pyta najpierw, gdy coś tam gra. |
| Test Main / Test Cue | Odtwarza krótki ton (1 kHz na Main, 440 Hz na Cue, 1,5 s, −18 dBFS) na wybranym wyjściu, abyś mógł sprawdzić okablowanie przed wejściem na antenę |
| Biblioteka | Wyjście Cue biblioteki, z własnym przyciskiem **Test Cue**: tam słychać [odsłuch wstępny](library.md#pre-listening) okna biblioteki (na telefonie: karty Biblioteka). Bez niego przycisk Odsłuch paska odsłuchu jest przyciemniony. W Androidzie jego pusty wybór brzmi *Domyślne systemowe* zamiast *Brak*: odsłuch wstępny i **Test Cue** grają wtedy na wyjściu domyślnym (zob. [Android → Biblioteka](android.md#library)). Wyjście Biblioteki na wyjściu antenowym działa jak Cue odtwarzacza: uwaga pod nim mówi, że jego odsłuch wstępny jest słyszalny na antenie, i pyta najpierw, gdy coś tam gra. Jest w widokach Podstawowe i Zaawansowane, także na telefonie |
| Cartwall | Wyjścia Main i Cue cartwalla. Main domyślnie jest wyjściem systemowym. Bez Cue nie ma odsłuchu wstępnego cartów; Cue na wyjściu antenowym działa jak w odtwarzaczach. |
| Częstotliwość próbkowania: *urządzenie* (Zaawansowane) | **Globalna (...)** używa częstotliwości powyżej; wartość nadaje temu urządzeniu własną. Oferowane są tylko częstotliwości zgłaszane przez urządzenie; zapisana częstotliwość, której już nie zgłasza, pozostaje na liście z uwagą, że może się nie otworzyć (urządzenie wraca wtedy do częstotliwości globalnej). Własne wartości dotyczą tylko urządzeń wskazanych przez wyjście, nie domyślnego wyjścia systemowego, chyba że któreś je wskazuje. |
| Rozmiar bufora: *urządzenie* (Zaawansowane) | **Globalny (...)** używa rozmiaru bufora powyżej (z minimum, o jakie prosi wyjście współdzielone, jak wyżej); wartość nadaje temu urządzeniu własny, używany bez zmian, z opóźnieniem pod spodem. Urządzenie, które nie przyjmuje własnego rozmiaru bufora, wraca do globalnego, a do globalnej częstotliwości także wtedy, gdy nie przyjmuje własnej. |
| Bit-perfect: *urządzenie* (Zaawansowane) | Domyślnie wyłączone. Specjalny tryb do odsłuchu audiofilskiego, nie do programów z miksami: urządzenie jest otwierane z dostępem na wyłączność i podąża za częstotliwością próbkowania każdego utworu. Odtwarzacze, których wyjście Main jest na nim, nigdy nie przenikają ani nie miksują, zmiana częstotliwości zostawia krótką ciszę między dwoma utworami, a ich głośność jest utrzymywana na 100 %. Przełącznik jest wyłączony tam, gdzie urządzenie nie może dać dostępu na wyłączność. Zob. [Jakie są koszty](bit-perfect.md#what-it-costs). |
| DSD: *urządzenie* (Zaawansowane) | **Konwertuj na PCM** (domyślnie), **DoP** lub, w Linuksie, **Natywne DSD**. Pokazuje je każde urządzenie; oferowane są tylko tryby, które urządzenie może przyjąć, a wiersz pod spodem mówi, dlaczego inne nie. Zob. [DSD](bit-perfect.md#dsd). |
| Gdy inne źródło potrzebuje wyjścia DSD (Zaawansowane) | **Kontynuuj utwór DSD jako PCM** (domyślnie) lub **Zachowaj DSD i wycisz inne źródła**. Zob. [DSD](bit-perfect.md#dsd). |
| Cisza DSD (Zaawansowane) | Cisza wysyłana przed rozpoczęciem strumienia DSD, po jego zakończeniu i przy przejściu na PCM, aby przetwornik zsynchronizował się bez trzasku; domyślnie 200 ms, od 0 do 2000. |

Wyjście Cue nigdy nie wraca do wyjścia, którego używa Main. Cue wskazujące
urządzenie w systemie audio, którego ten komputer nie ma, oznacza „brak cue”.
Cue na wyjściu antenowym (to samo urządzenie i kanały co Main albo
nakładające się na nie) jest dozwolone: uwaga pod nim mówi, że jego odsłuch
wstępny jest słyszalny na antenie i że najpierw pyta, gdy coś tam gra.
Odtwarzacz bez wyjścia Cue ma przyciemniony przycisk **CUE**; po najechaniu
kursorem podpowiada, aby wybrać tutaj wyjście Cue. To samo dotyczy **Odsłuchaj
na CUE** w cartwallu.

Jeśli urządzenie zniknie podczas odtwarzania, odtwarzacze zachowują swoje
osie czasu, a urządzenie jest otwierane ponownie, gdy wróci (zob.
[Rozwiązywanie problemów](troubleshooting.md)).

## Odtwarzacze {#players}

![Ustawienia, Odtwarzacze: liczba odtwarzaczy, tryb domyślny, czas wyciszania, automatyczny miks, cue-in i cue-out, ostrzeżenie o końcu utworu i język](../../images/guide/settings-players.png)

| Ustawienie | Domyślnie | Znaczenie |
|---|---|---|
| Język | Systemowy | Język interfejsu |
| Liczba odtwarzaczy | 4 | Kolumny na ekranie głównym (nie można usunąć odtwarzacza na antenie). Liczba obowiązuje po puszczeniu suwaka; liczba wpisana w pole obowiązuje po naciśnięciu Enter lub opuszczeniu pola, a Escape ją anuluje |
| Tryb domyślny | CONT | Tryb, w którym startują odtwarzacze |
| Czas wyciszania | 1000 ms | Używany przez Play na antenie i przez Stop z wyciszeniem. W odtwarzaczu, którego wyjście Main jest bit-perfect, tylko przez Stop z wyciszeniem |
| Automatyczny miks w punkcie MIX | Wł. | Nakłada utwory w trybie ciągłym. Nie dotyczy odtwarzacza, którego wyjście Main jest bit-perfect |
| Używaj cue-in i cue-out | Wł. | Wył.: odtwarzacze grają każdy utwór od początku do końca pliku; markery są zachowywane, a carty nadal używają swoich. Czasy trwania i sumy playlist podążają za tym samym zakresem |
| Ostrzeżenie o końcu utworu | 10 s | Kiedy odliczanie zaczyna migać na czerwono |

Gdy wyjście Main odtwarzacza jest bit-perfect, uwaga pod **Automatyczny miks w
punkcie MIX** mówi, że takie odtwarzacze nigdy nie miksują ani nie przenikają.
Oba ustawienia pozostają edytowalne: nadal dotyczą pozostałych odtwarzaczy. Zob.
[Jakie są koszty](bit-perfect.md#what-it-costs).

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

Progi opisane w rozdziale [Markery i miksowanie](markers-and-mixing.md);
każdy wiersz ma pod nazwą jednowierszową podpowiedź mówiącą, co robi wartość.
**Analizuj ponownie wszystkie utwory** uruchamia analizę od nowa dla całej
biblioteki, czytając każdy plik jeszcze raz; markery ręczne są zachowywane.
Gdy działa, przycisk jest przyciemniony i pokazuje, jak daleko zaszła
(**Analiza… 120 z 340**), więc ponowne naciśnięcie nie zaczyna od początku;
podpowiedź pozycji **Analiza N utworów** na pasku stanu podaje ten sam postęp.
Zmiana progu tutaj też analizuje bibliotekę od nowa i restartuje trwający
przebieg, tak aby żaden utwór nie zachował markerów ze starych wartości.

Po aktualizacji, w której zmieniła się analiza, utwory przeanalizowane przez
wcześniejszą wersję zachowują swoje markery i przebiegi, które nadal
działają. Przy starcie Fauste Player mówi, ile ich jest, i oferuje
**Analizuj teraz** lub **Później**; **Analizuj nieaktualne utwory (N)** tutaj
robi to samo w dowolnej chwili. Utwory w odtwarzaczach i tak są aktualizowane
w miarę wyświetlania, podobnie jak utwory cartów, które nie mają zapisanego
formatu (cart gra bit-perfect tylko wtedy, gdy jego format jest znany).
Utwory, których pliku brakuje, nie są liczone, dopóki plik nie wróci.

Analiza mierzy też głośność każdego pliku na potrzeby [normalizacji
głośności](loudness.md). Pliki przeanalizowane, zanim mierzono głośność, są
mierzone w tle, gdy analiza nie ma nic innego do roboty, niezależnie od tego,
czy normalizacja jest włączona; ich markery nie są ruszane. **Analizuj
ponownie** (w menu playlisty) i **Analizuj ponownie wszystkie utwory** mierzą
od nowa.

## Głośność {#loudness}

![Ustawienia, Głośność: normalizacja włączona, z wartością docelową, pułapem true peak, maksymalnym wzmocnieniem i maksymalną korektą w wartościach domyślnych](../../images/guide/settings-loudness.png)

Odtwarza każdy utwór i każdy cart z tą samą głośnością, ze stałym wzmocnieniem
dla każdego utworu. [Normalizacja głośności](loudness.md) wyjaśnia, jak
obliczane jest wzmocnienie, plakietkę w nagłówku odtwarzacza i ustawienia dla
każdego utworu.

| Ustawienie | Domyślnie | Zakres | Znaczenie |
|---|---|---|---|
| Normalizuj głośność | Wł. | | Normalizacja włączona lub wyłączona |
| Cel | −16 LUFS | −36 … −6 LUFS, krok 0,5 | Głośność zintegrowana, do której sprowadzany jest każdy utwór |
| Pułap true peak | −1 dBTP | −12 … 0 dBTP, krok 0,1 | Podbicie nigdy nie wynosi szczytu rzeczywistego utworu powyżej tej wartości. Obniżenia nie są ograniczane |
| Maksymalne wzmocnienie | 12 dB | 0 … 24 dB | Największe wzmocnienie, jakie może dostać cichy utwór. Obniżenia nie są ograniczane |
| Maksymalna korekta | 12 dB | 0 … 24 dB | Jak daleko może sięgać własna ręczna korekta każdego utworu, w górę lub w dół |

Zmiany dotyczą utworów wczytanych później: to, co gra, zachowuje swoje
wzmocnienie, a następny utwór czekający w każdym odtwarzaczu jest
przygotowywany od nowa z nowym. Uwaga pod ustawieniami mówi, ile utworów
wciąż czeka na pomiar; do jego zakończenia grają bez zmian. Odtwarzacz lub
cart na wyjściu bit-perfect nigdy nie jest normalizowany. **Przywróć
domyślne** resetuje tylko tę sekcję (normalizacja włączona). Normalizacja jest
włączona w nowych instalacjach; istniejąca konfiguracja zachowuje swoje
bieżące ustawienie (każda zapisana konfiguracja już przechowuje tę wartość).

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

Interfejs w każdym języku, także angielskim i hiszpańskim, został wykonany za
pomocą AI i może zawierać błędy. **O programie** zawiera ogólne zastrzeżenie
obejmujące je wszystkie. Poprawki są mile widziane jako zgłoszenia (issues)
lub pull requesty.

## Cartwall {#cartwall}

![Ustawienia, Cartwall: strony, siatka i edytor wybranego carta](../../images/guide/settings-cartwall.png)

Strony, rozmiar siatki, edytor cartów oraz import i eksport stron cartów.
Zob. [Cartwall](cartwall.md).

## Biblioteka {#library}

Twórz, zmieniaj nazwy, kolory i rodzaj, przenoś (przeciąganiem) i usuwaj
kategorie grupujące Twoje utwory, obserwuj folder za pomocą kategorii i usuwaj
jej brakujące utwory, bez otwierania okna biblioteki. Zob.
[Biblioteka](library.md#categories) i
[Obserwowane foldery](library.md#watched-folders).

## Skróty klawiszowe {#keyboard-shortcuts}

![Ustawienia, Skróty klawiszowe: każda akcja odtwarzacza z jej klawiszem i przycisk Odepnij obok przypisanych](../../images/guide/settings-shortcuts.png)

Zob. [Klawiatura](keyboard.md).

## MIDI {#midi}

![Ustawienia, MIDI, ze sterowaniem MIDI wyłączonym](../../images/guide/settings-midi.png)

Włącz powierzchnie sterujące MIDI, zobacz porty wejściowe (**Połącz porty
ponownie** otwiera je od nowa, tak jak ponowne podłączenie kontrolerów) i
naucz kontrolkę dla każdej akcji odtwarzacza. Zob. [Powierzchnie sterujące MIDI](midi.md).

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
