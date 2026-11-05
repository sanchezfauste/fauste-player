# Biblioteka

Biblioteka to wszystkie utwory, które zna Fauste Player: każdy plik dodany do
playlisty, carta lub samej biblioteki jest jednym utworem, ze swoimi tagami,
markerami i ustawieniami głośności. Okno biblioteki wymienia je, grupuje w
*kategoriach*, wyszukuje po tekście i wysyła do playlist i cartwalla. Na
komputerze i tablecie to okno; telefon pokazuje bibliotekę w swojej karcie
**Biblioteka** (zob. [Na telefonie](#on-a-phone)).

## Otwieranie okna {#opening-the-window}

Kliknij kafelek **Biblioteka** na górnym pasku. Świeci, gdy okno jest
otwarte; kliknij go ponownie albo przycisk zamknięcia okna, aby je zamknąć.
Okno unosi się nad odtwarzaczami i cartwallem i ich nie blokuje: nic w nim nie
zatrzymuje dźwięku ani skrótów głównego okna.

Przesuwaj je za pasek tytułu i zmieniaj jego rozmiar od krawędzi i rogów.
Pasek tytułu ma trzy przyciski: **Minimalizuj bibliotekę** (zostaje tylko
pasek tytułu, w miejscu okna albo u góry głównego obszaru, gdy okno jest
zmaksymalizowane; kliknij go ponownie, aby rozwinąć), **Maksymalizuj
bibliotekę** (okno wypełnia główny obszar; **Przywróć bibliotekę** go
przywraca) i **Zamknij bibliotekę**. Okno, które by się nie mieściło, po
zmniejszeniu głównego okna lub przeniesieniu go na mniejszy ekran, jest
przywracane do głównego obszaru.

Położenie i rozmiar okna, stan zminimalizowany lub zmaksymalizowany,
szerokość drzewa, kolumny, kolejność sortowania i zaznaczona kategoria są
zapisywane z sesją i wracają przy następnym uruchomieniu. Zapisywane jest też,
czy okno jest otwarte, ale nic przez to nie gra samo. Tekst w polu
wyszukiwania nie jest zapisywany.

## Kategorie {#categories}

Drzewo po lewej grupuje utwory. U góry **Wszystkie utwory**; potem Twoje
kategorie, które mogą zawierać podkategorie; na końcu **Bez kategorii**,
czyli utwory, które nie należą do żadnej kategorii. Każda pozycja pokazuje
swój kolor i liczbę: kategoria liczy swoje utwory i utwory swoich podkategorii.
Utwór należy najwyżej do jednej kategorii.

- **Kliknij** pozycję, aby wyświetlić jej utwory w tabeli. Kategoria
  wyświetla też utwory swoich podkategorii.
- **Strzałka** przed kategorią otwiera lub zamyka jej podkategorie. To, które
  są otwarte, jest zapisywane.
- **Nowa kategoria** (＋ nad drzewem) pyta o nazwę, kolor i *rodzaj*. Nowa
  kategoria przejmuje kolor i rodzaj od kategorii, w której jest tworzona;
  rodzaj to to, co przyjmuje utwór, gdy do niej dołącza, a późniejsza zmiana
  rodzaju zostawia utwory, które już w niej są, bez zmian. Nazwa nie może być
  pusta ani powtarzać nazwy rodzeństwa; kategorie zagnieżdżają się tylko do
  pewnej granicy.
- **Kliknij prawym przyciskiem** kategorię (albo przytrzymaj ją na ekranie
  dotykowym), aby uzyskać **Nowa podkategoria…**, **Zmień nazwę…**, **Kolor**,
  **Rodzaj** i **Usuń…**. Na komputerze menu ma też **Obserwuj folder…**
  (**Zmień obserwowany folder…** i **Przestań obserwować folder**, gdy
  obserwuje już jakiś) oraz **Usuń brakujące utwory…**; zob. [Obserwowane
  foldery](#watched-folders) i [Usuwanie utworów](#removing-tracks).
- **Przeciągnij** kategorię, aby ją przenieść: na inną kategorię, aby ją
  zagnieździć, albo w przerwę między dwiema kategoriami, aby ją tam umieścić.
  Kategoria nie może trafić do samej siebie. Na ekranie dotykowym drzewo
  przyjmuje tylko stuknięcia i przewija się pod palcem; kategorii nie można tam
  przeciągać.

**Usuń…** wymienia, co odchodzi razem z kategorią: jej podkategorie i każdy
utwór w nich są usuwane z biblioteki. Pliki nie są nigdy usuwane. Jeśli
któryś z tych utworów jest używany, drugie pytanie mówi, ile pozycji playlist
i cartów ich używa: **Usuń mimo to** usuwa pozycje z ich playlist i opróżnia
carty. Nic nie jest usuwane, dopóki któryś z nich jest na antenie; komunikat
mówi, co gra.

To samo drzewo jest w **Ustawienia → Biblioteka**, do tworzenia, zmiany nazwy i
koloru, zmiany rodzaju, przenoszenia i usuwania kategorii bez otwierania okna
(nie ma tam *Wszystkie utwory*, *Bez kategorii* ani zaznaczenia).

## Obserwowane foldery {#watched-folders}

Na komputerze kategoria może obserwować jeden folder. Kliknij ją prawym
przyciskiem i wybierz **Obserwuj folder…**, a potem wskaż folder. Kategoria
pokazuje znak ⟳ przed swoją liczbą; najedź na niego, aby odczytać folder.

- Każdy plik audio w folderze i jego podfolderach dołącza do kategorii.
  Podfoldery nie tworzą podkategorii.
- Utwór, który jest już w innej kategorii, tam zostaje. Utwór bez kategorii
  dołącza do tej. Ustawienie, zmiana lub zakończenie obserwacji nigdy nie
  przenosi utworu: utwory folderu, który nie jest już obserwowany, zostają w
  swojej kategorii.
- Gdy aplikacja działa, folder jest śledzony: nowy plik dołącza, gdy przez
  chwilę jest cicho (`library.watch_debounce_ms`, domyślnie 1 sekunda, więc plik,
  który wciąż jest kopiowany, nie jest czytany w połowie zapisu). Folder jest
  też skanowany przy każdym uruchomieniu aplikacji, aby wychwycić to, co
  zmieniło się, gdy była zamknięta.
- Plik, który znika z folderu (usunięty, przeniesiony lub przemianowany),
  zostawia swój utwór w bibliotece, oznaczony jako brakujący, z jego markerami.
  Jeśli plik wróci, utwór sam znów jest normalny. Przemianowany plik dołącza
  jako nowy utwór pod nową nazwą.
- Obserwowane foldery mogą być zagnieżdżone. Plik pod dwoma z nich trafia do
  kategorii najbardziej wewnętrznego; zakończenie obserwacji jednego zostawia
  drugi śledzony. Ten sam folder nie może być obserwowany przez dwie
  kategorie: aplikacja mówi, która kategoria już go ma.
- Nic, co robi obserwowany folder, nie wczytuje ani nie odtwarza utworu.

Jeśli folderu nie można śledzić, znak ⟳ zmienia kolor, a jego podpowiedź mówi
dlaczego: *Nie znaleziono obserwowanego folderu: …. Zostanie wyszukany
ponownie.* (na przykład niezamontowany dysk), *Nie można odczytać
obserwowanego folderu: …. Zostanie podjęta kolejna próba.* albo *Zmiany w …
nie są śledzone; folder jest skanowany przy starcie i podejmowana jest kolejna
próba.* (system odmówił obserwacji). Folder zapisany jako niepełna ścieżka
(edytowany ręcznie) nie jest w ogóle obserwowany, i mówi o tym znak.

Co 30 sekund (`tuning.missing_recheck_ms`) aplikacja sprawdza, czy każdy
obserwowany folder nadal jest, więc odłączony podczas pracy dysk też jest
zgłaszany. Folder w kłopotach jest ponawiany w tym tempie i skanowany, gdy
tylko wróci. Folder, którego nie ma, którego nie można odczytać albo który
jest pusty, podczas gdy biblioteka nadal ma z niego utwory (niezamontowany
dysk lub udział często wygląda jak pusty folder), liczy się jako nieznaleziony.
Obserwacja nie oznacza wtedy żadnego z jego utworów jako brakującego i niczego
nie usuwa, także gdy folder leży wewnątrz innego obserwowanego folderu.

Jeśli opróżnisz obserwowany folder celowo, ostrzeżenie zostaje, dopóki pliki
do niego nie wrócą albo nie przestaniesz go obserwować, a jego utwory zostają
w bibliotece bez oznaczenia jako brakujące. Aby je wyczyścić, zaznacz je w
tabeli i wybierz **Usuń z biblioteki…**, albo wybierz na nich **Analizuj
ponownie**: analiza oznacza utwory, których plik zniknął, jako brakujące, a
**Usuń brakujące utwory…** je wtedy usuwa (jeśli folder jest nadal
obserwowany, jego ostrzeżenie znika przy następnym sprawdzeniu).

Jeden wyjątek od powyższej reguły: utwór, który nigdy nie został
przeanalizowany, jest nadal oznaczany jako brakujący, gdy analiza nie może
znaleźć jego pliku, gdziekolwiek jest.

Podfolder, którego nie można odczytać, jest pomijany, a log o tym mówi;
dowiązania do folderów (dowiązania symboliczne do folderów) nie są śledzone.

Nowe pliki trafiają do analizy za wszystkim, czego potrzebuje odtwarzacz lub
cart, więc folder z tysiącami plików zajmuje chwilę; pokazuje to postęp
analizy.

To samo menu i znak są w **Ustawienia → Biblioteka**. Nie ma ich w Androidzie,
który nie obserwuje folderów: tam kategoria może brać pliki z folderu
urządzenia, który wczytujesz ponownie, gdy wybierzesz (zob. [Android →
Biblioteka](android.md#library)).

## Tabela utworów {#the-track-table}

Tabela wymienia utwory zaznaczonej pozycji, w tych samych kolumnach co tabele
playlist (wybranych w **Ustawienia → Playlisty → Kolumny tabeli**, bez
pozycji `#`), a po nich **Kategoria** i **Rodzaj**. Wąska komórka po lewej
oznacza utwór, którego plik jest brakujący lub nieczytelny. Utwór czekający na
analizę jest oznaczony po prawej stronie komórki **Tytuł** (więc tylko gdy
kolumna Tytuł jest pokazana).

- Przeciągnij separator nagłówka, aby zmienić szerokość kolumny; kliknij
  nagłówek, aby sortować według niego, i ponownie, aby odwrócić. Szerokości i
  kolejność sortowania są zapisywane.
- **Kliknięcie** zaznacza wiersz, **Ctrl** (**Cmd** w macOS) **kliknięcie**
  dodaje lub usuwa jeden, **Shift kliknięcie** zaznacza wiersze między ostatnim
  kliknięciem a tym. Zaznaczenie zachowuje tylko wiersze, które są wymienione:
  wiersz, który opuszcza tabelę z powodu wyszukiwania, innej kategorii lub
  usunięcia, opuszcza zaznaczenie. Każda akcja dotyczy zaznaczonych wierszy w
  kolejności tabeli.
- **Dodaj pliki** (nad tabelą) wybiera pliki audio i dodaje je do biblioteki, w
  zaznaczonej kategorii (bez kategorii, gdy zaznaczone jest *Wszystkie utwory*
  lub *Bez kategorii*).

Na ekranie dotykowym wiersze przyjmują tylko stuknięcia i przewijają się pod
palcem; przytrzymanie otwiera menu, a utwory wysyła się menu (niżej), nie
przeciąganiem.

## Wyszukiwanie {#search}

Pole nad tabelą wyszukuje utwory po tekście. Szuka w tytule, wykonawcy,
albumie, wykonawcy albumu, kompozytorze, gatunku, komentarzu i nazwie pliku,
ignorując wielkość liter i znaki diakrytyczne, a utwór jest wymieniany, gdy
zawiera każde wpisane słowo (w dowolnej kolejności i w dowolnym z tych pól).
Przeszukuje zaznaczoną pozycję; włącz **Cała biblioteka**, aby przeszukać każdy
utwór niezależnie od zaznaczenia. Zakres jest zapisywany; tekst nie. Wyczyść
pole, aby znów zobaczyć całą pozycję.

## Menu wiersza {#the-row-menu}

Kliknij wiersz prawym przyciskiem. Jeśli wiersz nie jest zaznaczony, staje się
zaznaczeniem; w przeciwnym razie menu działa na każdy zaznaczony wiersz.
Pierwszy wiersz pokazuje tytuł utworu, gdy menu dotyczy jednego utworu, oraz
ile utworów jest zaznaczonych, gdy dotyczy kilku.

| Pozycja | Działanie |
|---|---|
| Dodaj do ▸ | Playlista: utwory są dodawane na jej końcu |
| Dodaj do cartu ▸ | Cart pokazywanej strony cartwalla: utwory wypełniają wybrany cart i następne puste carty strony, w kolejności siatki. Utwory, które się nie mieszczą, są pomijane, a komunikat je zlicza |
| Przenieś do kategorii ▸ | Kategoria albo Bez kategorii |
| Rodzaj ▸ | Muzyka, Jingiel, Efekt, Reklama lub Głos |
| Edytuj tagi… | Edytor tagów (jeden utwór) |
| Głośność… | Okienko głośności (jeden utwór) |
| Analizuj ponownie | Analizuje utwory od nowa |
| Przywróć automatyczne znaczniki | Odrzuca markery ręczne |
| Pokaż lokalizację pliku | Otwiera folder pliku (jeden utwór) |
| Usuń z biblioteki… | Usuwa utwory z biblioteki, po zapytaniu (niżej) |

## Odsłuch wstępny {#pre-listening}

Pasek pod paskiem tytułu odsłuchuje wstępnie zaznaczony utwór na własnym
wyjściu **Biblioteka** biblioteki (wybieranym w [Ustawienia → Wyjścia
audio](settings.md#audio-outputs)), na przykład w słuchawkach. Telefon
odsłuchuje wstępnie ze swojej listy utworów (zob. [Na
telefonie](#on-a-phone)).

Pasek pokazuje:

- **Odsłuch**, **Wstrzymaj odsłuch** i **Zatrzymaj odsłuch**;
- duży czas pozostały utworu, jego tytuł, wykonawcę i kategorię;
- mały miernik po prawej. Odczytuje tylko ten odsłuch wstępny, nigdy miernik
  odtwarzacza ani cartwalla; kliknij go, aby wyzerować jego maksimum;
- przebieg całego pliku, z jego markerami, a pod nim wiersz
  `upłynęło / całość`.

Odsłuch rozpoczyna zaznaczony utwór od jego cue-in i gra go do końca pliku.
Odsłuch na wstrzymanym podglądzie go wznawia, a na grającym nic nie robi: Stop,
potem Odsłuch, zaczyna od cue-in. Normalizacja głośności działa tak jak na CUE
odtwarzacza.

Przebieg działa jak w oknie CUE odtwarzacza: kliknij go, aby przeskoczyć
(kliknięcie lub puszczenie myszy przeskakuje; przeciągnięcie nie), powiększaj
go i przesuwaj kółkiem oraz edytuj markery i otwieraj **Głośność…** z jego menu
(zob. [Edycja markerów](markers-and-mixing.md#editing-markers)). Gdy podgląd
jest zatrzymany, kliknięcie zamiast tego ustawia, skąd zacznie następny
Odsłuch.

**Podgląd podąża za zaznaczeniem.** Przy kilku zaznaczonych wierszach używa
ostatniego, który kliknąłeś.

- Gdy gra, zaznaczenie innego wiersza przełącza na ten utwór od jego cue-in i
  gra dalej, bez pytania.
- Gdy jest wstrzymany, nowy utwór jest wczytywany we wstrzymaniu.
- Gdy jest zatrzymany, zostaje zatrzymany, a pasek pokazuje nowy utwór.
- Zaznaczenie niczego (żaden wiersz nie jest zaznaczony albo zaznaczony wiersz
  został odfiltrowany) zostawia trwający podgląd bez zmian.
- Zaznaczenie utworu, którego nie można odtworzyć (jego plik jest brakujący
  lub nieczytelny), zatrzymuje podgląd, a pasek mówi, że pliku nie można
  odtworzyć.

**Co go zatrzymuje.** Stop; usunięcie jego utworu z biblioteki; zamknięcie
okna (minimalizacja nie). Nigdy nie gra po ponownym uruchomieniu.

**Bez wyjścia Biblioteka** (na komputerze; Android gra zamiast tego na
wyjściu domyślnym, zob. [Android → Biblioteka](android.md#library)) Odsłuch
jest przyciemniony (najedź na niego, aby zobaczyć przyczynę), a wiersz obok
mówi, że odsłuch wymaga wyjścia CUE biblioteki, z odnośnikiem otwierającym
Ustawienia → Wyjścia audio na wierszu **Biblioteka**. Pasek nadal pokazuje
utwór, jego przebieg i markery.

**Na wyjściu antenowym.** Wyjście Biblioteka może być wyjściem antenowym
(to samo urządzenie i kanały co Main odtwarzacza lub cartwalla). Działa wtedy
jak CUE odtwarzaczy i cartwalla (zob. [CUE (odsłuch
wstępny)](players.md#cue-pre-listen)): gdy coś tam gra, Odsłuch najpierw pyta
(**Odsłuchaj mimo to** albo **Anuluj**), a odsłuch wstępny, którego nie
potwierdziłeś, zatrzymuje się, gdy coś startuje na tym wyjściu.

## Przeciąganie do playlist i cartów {#dragging-to-playlists-and-carts}

Przeciągnij zaznaczone wiersze (albo wiersz, od którego zaczynasz
przeciąganie, gdy nie jest zaznaczony) myszą:

- na **kartę playlisty** odtwarzacza: utwory trafiają na koniec tej
  playlisty;
- na **tabelę playlisty**: utwory trafiają tam, gdzie pokazuje fioletowa
  linia;
- na **cart**: cart dostaje pierwszy utwór (cart zawiera jeden plik),
  zastępując swój plik, gdy jest zatrzymany. Cart, który gra lub jest na CUE,
  odmawia upuszczenia, ponieważ nowy plik zatrzymałby go na antenie.

## Usuwanie utworów {#removing-tracks}

**Usuń z biblioteki…** pyta najpierw: pliki nie są usuwane. Gdy któryś z
utworów jest używany, drugie pytanie zlicza pozycje playlist i carty, które je
tracą. Utwór, który jest na antenie (gra w odtwarzaczu lub na cartwallu), nie
może być usunięty: nic nie jest usuwane, a komunikat mówi, co gra.

**Usuń brakujące utwory…** (w menu kategorii, na dowolnej kategorii) usuwa
utwory tej kategorii i jej podkategorii, których plik jest brakujący i których
nie używa żadna pozycja playlisty ani cart. Pyta najpierw, podając ich liczbę;
żaden plik nie jest ruszany. Używane utwory są zachowywane, a kategoria bez
żadnego mówi o tym w komunikacie. Użyj tego do posprzątania po tym, jak pliki
opuściły obserwowany folder: aplikacja nigdy nie usuwa ich sama.

## Z playlisty {#from-a-playlist}

Na komputerze menu utworu w wierszu playlisty ma **Pokaż w bibliotece** i
**Przenieś do kategorii ▸**, po **Głośność…**. *Pokaż w bibliotece* otwiera
okno na kategorii utworu i zaznacza jego wiersz; *Przenieś do kategorii*
przypisuje utwór bez otwierania okna. Dla utworu, którego nie ma w
bibliotece, *Pokaż w bibliotece* jest pokazane jako wyłączone, a *Przenieś do
kategorii* nie jest pokazywane.

## Na telefonie {#on-a-phone}

Telefon nie ma okna biblioteki: jego karta **Biblioteka** otwiera się na
kategoriach (*Wszystkie utwory*, Twoje kategorie, *Bez kategorii* i **Muzyka
na urządzeniu**), a stuknięcie jednej wyświetla jej utwory z wyszukiwaniem i
**Cała biblioteka**. Zaznacz utwory, aby dodać je do playlisty, przytrzymaj
wiersz, aby zobaczyć jego menu, odsłuchaj utwór jego 🎧 i dodaj muzykę
urządzenia do kategorii przyciskiem **＋** jej listy. W Androidzie, bez wyjścia
Biblioteka, odsłuch wstępny gra na wyjściu domyślnym. Zob. [Android →
Biblioteka](android.md#library).
