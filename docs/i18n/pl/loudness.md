# Normalizacja głośności

Utwory pochodzą z wielu źródeł i są masterowane na bardzo różnych poziomach:
niedawny singiel popowy może brzmieć dwa razy głośniej niż nagranie z lat 70.
albo mówiony zwiastun. **Normalizacja głośności** odtwarza każdy utwór i
każdy cart z tą samą odczuwalną głośnością, więc operator nie musi poprawiać
suwaka między jednym utworem a drugim.

Jest **włączona w nowych instalacjach**; istniejąca konfiguracja zachowuje
swoje bieżące ustawienie (każda zapisana konfiguracja już je przechowuje).
Zmienisz ją w [Ustawienia → Głośność](settings.md#loudness).

![Ustawienia, Głośność: normalizacja włączona, z wartością docelową, pułapem true peak, maksymalnym wzmocnieniem i maksymalną korektą w wartościach domyślnych](../../images/guide/settings-loudness.png)

## Jak to działa {#how-it-works}

**Stałe wzmocnienie dla każdego utworu.** Każdy utwór dostaje jedno
wzmocnienie, w dB, ustalane przy wczytaniu utworu i utrzymywane do jego
zatrzymania. Nic nie jest kompresowane ani limitowane: cały utwór jest
głośniejszy lub cichszy o tę samą wartość, więc jego własna dynamika zostaje
taka, jak go zamasterowano. Suwak odtwarzacza działa na tym wzmocnieniu, jak
dotąd.

**Pomiar.** Analiza w tle mierzy każdy plik raz:

- jego **głośność zintegrowaną**, w LUFS: jak głośno brzmi cały plik, według
  definicji ITU-R BS.1770 i EBU R128 (ciche fragmenty i cisza są odrzucane
  bramką, więc długie wyciszenie na końcu lub pauza jej nie obniżają);
- jego **szczyt rzeczywisty (true peak)**, w dBTP: najwyższy poziom, jaki
  sygnał osiąga między próbkami po przekształceniu na analogowy.

Nowe pliki są mierzone przy pierwszej analizie. Pliki dodane, zanim mierzono
głośność, są mierzone w **przebiegu w tle**, który działa, gdy zwykła analiza
nie ma nic do roboty, bez ruszania ich markerów. Działa niezależnie od tego,
czy normalizacja jest włączona, więc pomiary są gotowe, gdy ją włączysz.
**Analizuj ponownie** (w menu playlisty) mierzy plik od nowa, czytając go,
nawet jeśli jego rozmiar i data się nie zmieniły. Plik krótszy niż 0,4 s nie
jest nigdy mierzony, a plik cichy nie ma głośności: oba grają bez zmian.

**Wzmocnienie.** Wzmocnienie sprowadza zmierzoną głośność do wartości
**Cel**, plus własna **Korekta** utworu (zob. [Dla każdego
utworu](#per-track-exclude-and-trim)):

> wzmocnienie = cel − zmierzona głośność + korekta

Utwór głośniejszy niż cel jest ściszany o tyle, ile trzeba. Cichszy utwór jest
podgłaśniany, ale nigdy o więcej niż **Maksymalne wzmocnienie** i nigdy tak,
by jego szczyt rzeczywisty przekroczył **Pułap true peak**: nie ma limitera,
więc to pułap chroni podbity utwór przed przesterowaniem. Utwór, którego
szczyt rzeczywisty już przekracza pułap, nie jest podbijany wcale.

Przykłady, przy wartościach domyślnych (cel −16 LUFS, pułap −1 dBTP,
maksymalne wzmocnienie 12 dB):

| Utwór | Zmierzono | Szczyt rzeczywisty | Wzmocnienie |
|---|---|---|---|
| Głośny nowoczesny master | −9,0 LUFS | +0,5 dBTP | −7,0 dB |
| Cichszy starszy master | −20,0 LUFS | −10,0 dBTP | +4,0 dB |
| Cichy utwór z wysokimi szczytami | −22,0 LUFS | −3,0 dBTP | +2,0 dB (ograniczone pułapem) |
| Bardzo ciche nagranie | −35,0 LUFS | −20,0 dBTP | +12,0 dB (ograniczone maksymalnym wzmocnieniem) |

**Kiedy wzmocnienie jest ustalane.** Przy wczytaniu utworu: przygotowanym jako
następny w odtwarzaczu, uruchomionym, wczytanym we wstrzymaniu, odsłuchanym
wstępnie na CUE albo odpalonym jako cart. Utwór, który gra lub jest
wstrzymany, **zachowuje swoje wzmocnienie** do zatrzymania, mimo przeskoków i
pauz. Zmiana ustawień, pomiaru utworu lub jego korekty obowiązuje przy
następnym wczytaniu utworu; następny utwór, który już czeka w odtwarzaczu, jest
przygotowywany od nowa z nowym wzmocnieniem, więc startuje z nim.

**Odsłuch wstępny.** CUE i odsłuch wstępny carta odtwarzają utwór z tym samym
wzmocnieniem, jakie dostaje na antenie, więc to, co słyszysz w słuchawkach,
zgadza się z programem.

**Mierniki.** Mierniki i ich odczyt głośności mierzą po wzmocnieniu, więc
pokazują poziom znormalizowany. **Docelowa głośność** samych mierników
(Ustawienia → Mierniki) tylko koloruje odczyt; nie zmienia żadnego
wzmocnienia.

## Wyjścia bit-perfect {#bit-perfect-outputs}

Odtwarzacz lub cart, którego wyjście jest [bit-perfect](bit-perfect.md), **nie
jest nigdy normalizowany**: zawsze gra ze wzmocnieniem jednostkowym (0 dB),
niezależnie od ustawień, ponieważ każde wzmocnienie zmieniłoby próbki, które
otrzymuje urządzenie. Przy włączonej normalizacji jego nagłówek pokazuje
bursztynową plakietkę **0.0 dB**, której podpowiedź mówi „Wyjście
bit-perfect: utwory grają bez zmian.” To samo dotyczy odsłuchu wstępnego na
urządzeniu Cue bit-perfect, podczas gdy wyjście Main odtwarzacza, jeśli nie
jest bit-perfect, nadal dostaje wzmocnienie.

Utwór, który już gra (lub jest wstrzymany), gdy jego urządzenie staje się
bit-perfect, zachowuje wzmocnienie, z jakim wystartował, a plakietka **BP**
pozostaje dla niego wyłączona. Następny utwór gra z 0 dB i bit-perfect. Utwór
DSD w takiej sytuacji gra skonwertowany do PCM ze swoim wzmocnieniem, nigdy
jako niezmieniony strumień DSD.

## Wzmocnienie w nagłówku odtwarzacza {#the-gain-in-the-player-header}

Gdy odtwarzacz gra lub jest wstrzymany, plakietka na lewo od **BP** pokazuje
wzmocnienie utworu na antenie. Plakietka ma obramowanie w swoim kolorze i
zawsze tę samą szerokość. Najedź na nią kursorem, aby zobaczyć przyczynę.

| Plakietka | Znaczenie |
|---|---|
| **+4.0 dB**, **−7.0 dB** (niebieska) | Utwór jest normalizowany tym wzmocnieniem. Podpowiedź to zestawienie wzmocnienia opisane niżej. Gdy ustawienia, pomiar lub korekta zmieniły się po wczytaniu utworu, mówi tylko, że utwór zachowuje wzmocnienie ustalone przy wczytaniu, i pokazuje to wzmocnienie. |
| **0.0 dB** (bursztynowa) | Utwór gra bez zmian. Podpowiedź to to samo zestawienie, a jego wiersz **Wzmocnienie automatyczne** mówi dlaczego: nie ma jeszcze pomiaru (albo jest za krótki do pomiaru), wyjście jest bit-perfect albo utwór jest wyłączony z normalizacji. |
| Brak plakietki | Normalizacja była wyłączona, gdy utwór startował, utwór jest cichy albo odtwarzacz jest zatrzymany. Odtwarzacz bit-perfect też nie pokazuje jej, gdy normalizacja jest wyłączona. |

Plakietka pokazuje wzmocnienie, o jakie odtwarzacz poprosił, gdy utwór
startował.

### Zestawienie wzmocnienia {#the-gain-breakdown}

Wszędzie tam, gdzie program wyjaśnia wzmocnienie utworu, pokazuje ten sam blok
wierszy, z etykietą po lewej i wartością po prawej: podpowiedź plakietki
wzmocnienia, okienko głośności (pod jego kontrolkami), edytor carta i
podpowiedź utworu w tabeli playlisty.

| Wiersz | Znaczenie |
|---|---|
| **Głośność zintegrowana** | Zmierzona głośność utworu, w LUFS (albo „niezmierzona”). |
| **Szczyt rzeczywisty** | Jego najwyższy szczyt rzeczywisty, w dBTP (tylko gdy zmierzony). |
| **Cel** | Głośność, do której sprowadzany jest każdy utwór (Ustawienia → Głośność). |
| **Wzmocnienie automatyczne** | Cel minus zmierzona głośność, przed korektą i jakimkolwiek limitem, albo „brak” i dlaczego: wykluczony, wyjście bit-perfect, normalizacja wyłączona, jeszcze niezmierzony albo cichy. |
| **Korekta** | Używana ręczna korekta utworu, gdy nie wynosi 0 (z zapisaną wartością i maksimum, gdy ogranicza ją obniżona **Maksymalna korekta**). |
| **Żądane wzmocnienie** i **Ograniczone przez** | Tylko gdy pułap true peak lub maksymalne wzmocnienie powstrzymały wzmocnienie: wzmocnienie automatyczne plus korekta (pokazywana, gdy jest korekta) i limit, który je zatrzymał. |
| **Wzmocnienie wynikowe** | Zawsze ostatni, pod linią i pogrubiony: wzmocnienie, z jakim utwór gra. 0.0 dB, gdy nie jest normalizowany. |

Przy wyłączonej normalizacji uwaga nad wzmocnieniem wynikowym mówi, że na
antenie nic się nie zmienia i że korekta zadziała po jej włączeniu.

## Dla każdego utworu: wyłączenie i korekta {#per-track-exclude-and-trim}

Niektóre utwory nie powinny podążać za celem: sygnał stacji zamasterowany
celowo głośno, ton testowy, utwór, którego pomiar nie zgadza się z tym, jak
brzmi (mowa często brzmi głośniej niż muzyka przy tym samym LUFS). Każdy
utwór ma na to dwa ustawienia:

- **Wyłącz z normalizacji:** utwór zawsze gra bez zmian.
- **Korekta:** dB dodawane do wzmocnienia utworu, od − do + **Maksymalna
  korekta** (Ustawienia → Głośność), w krokach co 0,5 dB. Kliknij suwak
  dwukrotnie, aby przywrócić 0. Korekta jest przyciemniona, gdy utwór jest
  wyłączony.

Suwak to **korekta**: przesunięcie na wzmocnieniu automatycznym, więc
pokazuje 0 dB dla każdego utworu, którego nie edytowałeś. Okienko i edytor
carta pokazują więc [zestawienie wzmocnienia](#the-gain-breakdown) ze swoimi
kontrolkami: wiersz **Wzmocnienie wynikowe** podąża za suwakiem, gdy nim
poruszasz. W okienku kontrolki są pierwsze, a zestawienie pod nimi, więc
suwak zostaje tam, gdzie jest, gdy zestawienie rośnie lub się kurczy. Gdy
utwór brzmi, okienko dodaje, że zmiana obowiązuje od następnego wczytania.

Gdzie je zmienić:

- **Na przebiegu:** kliknij prawym przyciskiem przebieg odtwarzacza (lub jego
  okna CUE) i wybierz **Głośność…**. Okienko podaje nazwę utworu i pokazuje
  jego pomiar. Nadal edytuje ten utwór, nawet gdy odtwarzacz przejdzie do
  innego; zamknij je przyciskiem **×**. Zamyka się samo, jeśli utwór zostanie
  usunięty.
- **W wierszu playlisty:** kliknij wiersz prawym przyciskiem i wybierz
  **Głośność…**: to samo okienko, dla utworu tego wiersza.
- **W edytorze carta:** Ustawienia → Cartwall, kliknij cart: te same dwie
  kontrolki są pod **Zatrzymuj inne carty po odpaleniu**, z pomiarem pod
  etykietą.
- **Zdalnie:** `PUT /api/v1/tracks/<id>/loudness` (zob. [Zdalne
  sterowanie](remote-control.md)).

Ustawienia należą do utworu i są z nim zapisywane. Zmiana obowiązuje przy
następnym wczytaniu utworu, jak każda inna zmiana.

**Podpowiedź utworu** w tabeli playlisty pokazuje tagi utworu i, pod linią,
to samo [zestawienie wzmocnienia](#the-gain-breakdown), obliczone dla
odtwarzacza, do którego należy ta lista (odtwarzacz bit-perfect gra go bez
zmian).

## Wybór ustawień {#choosing-the-settings}

- **Cel.** −16 LUFS pasuje do większości programów muzycznych na antenie i w
  sieci. −23 LUFS (EBU R128) lub −24 LUFS (ATSC A/85) zostawiają więcej zapasu,
  ale ciche utwory wymagają wtedy większych podbić, które pułap i maksymalne
  wzmocnienie mogą powstrzymać. Wybierz cel bliski temu, jak głośny jest już
  Twój materiał.
- **Pułap true peak.** −1 dBTP zostawia miejsce na konwersję i na stratne
  enkodery dalej w torze. Obniż go, gdy enkoder strumienia przesterowuje.
- **Maksymalne wzmocnienie.** Chroni bardzo ciche nagranie (albo prawie ciche)
  przed podbiciem do poziomu szumu. 0 dB tylko ścisza głośne utwory.
- **Maksymalna korekta.** Jak daleko może sięgać korekta każdego utworu.
  Obniżenie jej ogranicza korekty już ustawione, bez ich zmiany, aż do
  następnej edycji głośności tego utworu: edycja zapisuje wtedy ograniczoną
  korektę.

## Normy stojące za wartościami domyślnymi {#standards-behind-the-defaults}

- **Pomiar.** Głośność zintegrowana i szczyt rzeczywisty według definicji
  ITU-R BS.1770 (ważenie K, bramkowanie).
- **Cel −16 LUFS.** Podąża za zaleceniami AES dla streamingu i dystrybucji
  online. AES TD1008 (2021) zastępuje wcześniejszy AES TD1004.1.15-10, który
  zalecał od −16 do −20 LUFS dla streamingu. To nie jest wartość
  radiofoniczna: EBU R128 używa −23 LUFS, a ATSC A/85 −24 LKFS. Użyj jednej z
  nich, gdy wymaga tego specyfikacja dostawy (EBU R128 dla europejskiej
  wymiany programów radiowych i telewizyjnych, ATSC A/85 dla telewizji
  cyfrowej w USA), a −16 dla strumienia lub programu online.
- **Pułap true peak −1 dBTP.** Maksymalny szczyt rzeczywisty według EBU R128,
  używany też w zaleceniach AES. Stratne enkodery mogą przekraczać poziom przy
  dekodowaniu, więc strumień wysyłany przez enkoder o niskiej przepływności
  może wymagać niższego pułapu, takiego jak −2 dBTP (częsta praktyka, nie
  część tych norm).
- **Maksymalne wzmocnienie i maksymalna korekta, 12 dB.** Wybory produktowe,
  za którymi nie stoi żadna norma: dość, by podnieść ciche nagranie i
  skorygować utwór na słuch, nie tyle, by podbić szum albo by korekta
  przekreślała cel.
- **Normalizacja jest włączona w nowych instalacjach.** Aktualizacja
  aplikacji zachowuje ustawienie, jakie ma Twoja konfiguracja, więc
  instalacja, która miała ją wyłączoną, zostaje wyłączona, dopóki jej nie
  włączysz.
- **Docelowa głośność miernika** (Ustawienia → Mierniki) to osobne
  ustawienie, domyślnie wartość R128 −23 LUFS. Normalizacja jej nie używa.
