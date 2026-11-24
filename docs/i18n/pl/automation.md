# Automatyzacja

Automatyzacja utrzymuje stację na antenie, gdy nikogo nie ma w studiu. Ta
wersja ma **wypełniacz muzyczny**, odtwarzacz, który sam wybiera muzykę
według ustawionych przez Ciebie reguł i nigdy nie staje z braku listy, oraz
kolumnę **Ramówka**, która odtwarza bloki dnia o ustalonych godzinach,
zbudowane z szablonów w Ustawieniach.
Odtwarzacze, których już używasz (P1 … Pn), pozostają nietknięte: nadal są
Twoje, do audycji, które przygotowujesz i grasz na żywo.

## Kolumna Ramówka {#the-log-column}

Kolumna **Ramówka** odtwarza **bloki** dnia: grupy elementów z godziną
rozpoczęcia (sygnał godzinowy, spoty, audycja). Pokaż ją kafelkiem
**Ramówka** na górnym pasku, tak jak odtwarzacz.

![Działająca kolumna Ramówka obok kolumny Wypełniacz muzyczny: bloki dnia według godziny rozpoczęcia, każdy z sygnałem stacji (zegar oznacza go jako element o stałej godzinie), spotami i rotacją, którą wypełniacz muzyczny gra po nim; utwór wypełniacza muzycznego na antenie i następne](../../images/guide/automation-columns.png)

Jej lista wygląda jak playlista, pogrupowana według bloków. Każdy blok
zaczyna się wierszem nagłówka: godziną rozpoczęcia, liczbą elementów i ich
łącznym czasem, napisem „zmienione na ten dzień”, gdy zmieniłeś go dzisiaj,
oraz, na bursztynowo, o ile wchodzi na następny blok. Potem idą jego
elementy, z **Godziną**, o której każdy ma się zacząć. Lista pokazuje
dzisiaj, ostatni blok z wczoraj i pierwszy z jutra. Element na antenie i
następny są oznaczone jak w odtwarzaczu; elementy, które już poszły na
antenę, są przyciemnione, a element, który nie poszedł (**nienadany**), jest
przekreślony. Zegar oznacza element o **stałej godzinie**, który gra tylko o
swojej porze. **Utwór z kategorii** pokazuje wybrany dla niego utwór, gdy
tylko staje się następny (albo w swoim bloku przychodzi po następnym), z
kategorią za nim: to ten utwór zagra. Jeśli kategoria nie ma żadnego utworu,
który może zagrać (żaden nie jest do niej przypisany albo brakuje ich plików),
wiersz mówi o tym na bursztynowo, a element nie zostanie nadany. Element,
którego plik okaże się nieczytelny tuż przed odtworzeniem, jest pomijany i
również oznaczany jako nienadany. Utwory przypisuje się do kategorii w
bibliotece. Wiersz **wypełniacza muzycznego** mówi, do kiedy gra, i jest
pokazany jako na antenie, gdy wypełniacz muzyczny gra dla Ramówki. Następny,
którego godzina minęła, pokazuje ją na bursztynowo. Przeciągnij linię między
dwoma nagłówkami kolumn, aby zmienić szerokość kolumn, jak na playliście;
okna szablonu i dnia używają tych samych szerokości.

### Zmiana dzisiejszej ramówki {#changing-todays-log}

Ramówkę zmienia się jak playlistę, a zmiana dotyczy tylko tego dnia: szablon
zostaje bez zmian.

- Przyciski na dole kolumny: **+ Utwór…** otwiera bibliotekę, aby przeciągać
  utwory na listę; **+ Utwór z kategorii…** i
  **+ Wypełniacz muzyczny (rotacja)…** pytają, który, i dodają go tuż pod
  zaznaczonym wierszem (kliknij wiersz, aby go zaznaczyć; zaznacz nagłówek
  bloku, aby dodać na początku tego bloku), a gdy nic nie jest zaznaczone lub
  pod wierszem, którego nie można już zmienić, na końcu nadchodzącego bloku;
  **+ Blok…** pyta o godzinę, z wpisaną już pierwszą wolną. Gdy kolumna jest
  za wąska na wszystkie, te, które się nie mieszczą, są pod **⋯**.
- Przeciągnij utwory z biblioteki albo pliki audio z menedżera plików na
  wiersz, aby wstawić je przed nim (albo za nim, na dolnej połowie wiersza,
  jak na playliście: tak trafiają na koniec bloku), i przeciągaj wiersze w
  obrębie tego samego dnia, aby je przenieść. Dodany wiersz zostaje
  zaznaczony i przewinięty do widoku, więc następny, który dodasz, trafi pod
  nim.
- Kliknij wiersz prawym przyciskiem, aby wybrać **Stała godzina** i **Usuń z
  dzisiejszej ramówki**. Kliknij (lub kliknij prawym przyciskiem) nagłówek
  bloku, aby dodać na końcu tego bloku **Utwór…**, **Utwór z kategorii…** lub
  **Wypełniacz muzyczny (rotacja)…**, albo wybrać **Zmień godzinę…**, **Usuń
  blok** i, w zmienionym bloku, **Wróć do szablonu**.
- Gdy przeciągasz nad nagłówkiem bloku, linia pod nim pokazuje, że upuszczenie
  trafi na początek tego bloku; aby wstawić wiersze na koniec bloku powyżej,
  upuść je na dolnej połowie jego ostatniego wiersza.
- Tego, co już zagrało, elementu na antenie i minionych dni nie można
  zmienić. Zmiana bloku na antenie zaczyna działać po elemencie na antenie.

Kolumna Ramówka ma nagłówek i przyciski odtwarzacza i działają one jak w
odtwarzaczu:

- **Odtwarzaj** odtwarza element oznaczony jako następny i uruchamia
  automatyzację: każdy blok gra wtedy o swojej porze, a wypełniacz muzyczny
  gra między blokami. Gdy następny wybrał zegar, a jego blok zaczyna się
  później, do tego czasu gra muzyka. Nic nie startuje samo: po każdym
  uruchomieniu aplikacji automatyzacja jest zatrzymana, dopóki nie naciśniesz
  Odtwarzaj.
- **Pauza**, **Stop** i **Stop z wyciszeniem** działają na to, co jest na
  antenie dla Ramówki (jej element albo, między blokami, utwór wypełniacza
  muzycznego), i zatrzymują automatyzację. Odtwarzaj po Pauzie wznawia od
  miejsca, w którym było; w czasie pauzy następny się nie przesuwa, a po
  wznowieniu następny blok zachowuje swoją godzinę. Odtwarzaj po Stopie w
  czasie muzyki znów puszcza muzykę, a następny blok zaczyna się o swojej
  porze.
- **Stop po** pozwala dokończyć to, co jest na antenie, a potem automatyzacja
  się zatrzymuje. Jeśli gra muzyka, a wcześniej nadejdzie godzina bloku, blok
  zaczyna się punktualnie, a automatyzacja zatrzymuje się po jego pierwszym
  elemencie.
- **Poprzedni utwór** nic nie robi w Ramówce.
- **SINGLE** czyni Ramówkę ręczną: Odtwarzaj odtwarza następny element i
  zatrzymuje się po nim; żaden blok nie zaczyna się sam.

Gdy wypełniacz muzyczny gra między blokami, nagłówek Ramówki pokazuje
**Wypełniacz muzyczny · *rotacja*** (rotację, którą gra), a jej przyciski
nadal działają; utwór, jego miernik i przebieg są tylko w kolumnie
Wypełniacz muzyczny.

Gdy automatyzacja zatrzyma się po elemencie, następny zostaje na elemencie
pod tym, który zagrał, a Odtwarzaj go odtwarza, niezależnie od godziny: Ty
decydujesz. Gdy zatrzyma się w czasie muzyki, Odtwarzaj znów puszcza muzykę
do następnego bloku, chyba że wybrałeś inny następny. Aby zmienić, co zagra
dalej, kliknij dwukrotnie element albo kliknij go prawym przyciskiem i wybierz
**Zagraj jako następne**. **Wróć do zegara** w tym samym menu ustawia jako
następny to, co wynika z godziny.

## Szablony i dni {#templates-and-days}

Ramówka gra każdy dzień według **szablonu**: dnia złożonego z bloków. Nowa
instalacja ma jeden, „Podstawowy”, na każdy dzień. Szablony tworzy się w
Ustawienia → **Automatyzacja**, w sekcji **Szablony i dni** (nie w
Androidzie); kolumna Ramówka zmienia tylko dzisiejszy dzień, jak opisano
wyżej.

![Ustawienia, Automatyzacja, Szablony i dni: szablony Podstawowy i Weekend z Edytuj…, Nowy szablon, Duplikuj, Usuń i nazwą; tydzień wzorcowy, Podstawowy od poniedziałku do piątku i Weekend w sobotę i niedzielę; miesiąc z wybranym dniem dzisiejszym, daty ze zmianami na bursztynowo, szablon daty i Edytuj dzień…](../../images/guide/settings-plans.png)

- **Szablony.** Wybierz jeden i naciśnij **Edytuj…** (albo kliknij go
  dwukrotnie), aby go otworzyć; **Nowy szablon**, **Duplikuj** i **Usuń** są
  obok, a jego nazwa poniżej.
- **Tydzień wzorcowy.** Którego szablonu używa każdy dzień tygodnia.
- **Dni.** Wybierz datę w kalendarzu (daty ze zmianami są bursztynowe). Data
  może używać innego szablonu niż jej dzień tygodnia (święto w czwartek może
  używać weekendowego). **Edytuj dzień…** otwiera tę datę, aby zmienić jej
  bloki tylko na tę datę; minionych dat nie można zmieniać.

Szablon lub data otwiera się we własnym oknie, które zamyka Ustawienia. To ta
sama lista co kolumna Ramówka i zmienia się ją tak samo: przyciski na dole
(Utwór…, Utwór z kategorii…, Wypełniacz muzyczny (rotacja)…, Blok…, które
dodają pod zaznaczonym wierszem), utwory przeciągnięte z biblioteki (lub pliki
z menedżera plików) na wiersz, wiersze przeciągane, aby je przenieść, menu
prawego przycisku wierszy i menu nagłówków bloków. Nigdy nie gra i może
zostać otwarte obok biblioteki. Okno dzisiejszego dnia pokazuje, co zagrało i
co gra, jak Ramówka; okno dnia zamyka się o północy, gdy tego dnia nie można
już zmienić.

![Szablon otwarty w swoim oknie: bloki szablonu Podstawowy według godziny rozpoczęcia, każdy z sygnałem stacji, spotami i po nich wypełniaczem muzycznym, oraz Dodaj na dole](../../images/guide/template-popup.png)

**Utwór z kategorii** w szablonie gra jeden utwór z tej kategorii, za każdym
razem inny. Element **Wypełniacz muzyczny** wybiera rotację, którą wypełniacz
muzyczny gra po bloku (domyślną rotację, gdy blok nie ma żadnej). Gdy
elementy bloku trwają dłużej niż czas do następnego bloku, jego nagłówek mówi
na bursztynowo, o ile; mimo to możesz to tak zostawić.

Zmiana daty jest oznaczona na jej bloku napisem „zmienione na ten dzień”, a
**Wróć do szablonu** w menu nagłówka bloku ją cofa. Późniejsze zmiany
szablonu nadal docierają do bloków, których w tej dacie nie zmieniłeś. Zmiana
szablonu dokonana, gdy jeden z jego bloków jest na antenie, zostawia ten blok
na dziś bez zmian (pokazuje „zmienione na ten dzień”); zmiana dociera do niego
od następnego dnia.

## Wypełniacz muzyczny {#the-music-fill}

Wypełniacz muzyczny to kolumna **Wypełniacz muzyczny**. Jest ukryta, dopóki
jej nie pokażesz: kliknij kafelek **Wypełniacz muzyczny** po lewej stronie
górnego paska. Leży na lewo od P1.

Wygląda i działa jak odtwarzacz: duży zegar, przebieg, przyciski transportu,
suwak i lista utworów. Różnica to lista:

- **wypełnia się sama** z **rotacji** (poniżej), zawsze około pół godziny do
  przodu (*wyprzedzenie*, w Ustawienia → Automatyzacja);
- **sama się przycina**: zachowuje ostatnie 50 zagranych utworów i usuwa
  starsze, więc może działać tygodniami;
- przed blokiem ramówki może **zamienić utwory, które sama wybrała**, na
  inne, których długość pasuje do pozostałego czasu (zob. niżej); utworów
  dodanych przez Ciebie nigdy nie rusza;
- nadal możesz ją **zmieniać ręcznie**: przeciągać utwory z biblioteki,
  zmieniać kolejność wierszy, usunąć niechciany utwór. Dodany przez Ciebie
  wiersz jest oznaczony ręką i nigdy nie jest zastępowany; usunięty wiersz nie
  liczy się jako zagrany.

Jej wyjścia ustawia się w Ustawienia → Wyjścia audio, w wierszu **Wypełniacz
muzyczny**, jak dla każdego odtwarzacza. Nie ma numeru, skrótów klawiszowych
ani przypisań MIDI, a zdalne sterowanie jej nie widzi.

Naciśnij **Odtwarzaj**, aby go uruchomić, i **Stop po**, aby utwór na antenie
się skończył i kolumna się zatrzymała. Jak każdy odtwarzacz, nigdy nie
startuje sam, nawet po ponownym uruchomieniu. Gdy gra dla Ramówki między
blokami, zatrzymanie go lub pauza zatrzymuje też automatyzację, a jego Stop po
zatrzymuje automatyzację, gdy utwór się skończy; jeśli wcześniej nadejdzie
godzina bloku, blok gra, a oznaczenie znika. Po zapauzowaniu Ramówki w czasie
muzyki Odtwarzaj w kolumnie Wypełniacz muzyczny gra utwór samodzielnie:
Odtwarzaj Ramówki już go nie wznawia, a jej Stop już go nie zatrzymuje.

Nad jego listą, tam gdzie odtwarzacz ma karty playlist, kolumna Wypełniacz
muzyczny pokazuje **rotację**, którą gra. Wybierz tam inną: utwory, które
miała w kolejce, są zastępowane, te dodane przez Ciebie zostają. Gdy rotacja
nie znajduje nic do odtworzenia (jej kategorie są puste albo każdy plik jest
brakujący), pasek o tym mówi.

## Rotacje {#rotations}

**Rotacja** to krótka lista **kroków**, które wypełniacz muzyczny gra po
kolei, a potem znowu od pierwszego. Każdy krok mówi, skąd pochodzą jego
utwory (kategoria biblioteki z jej podkategoriami albo każdy utwór danego
rodzaju) i ile ich gra za każdym razem, jako zakres:

| Krok | Gra |
|---|---|
| Aktualne hity | 2–3 |
| Sygnały stacji | 1 |
| Złote przeboje | 0–1 |

Przy zakresie sygnał stacji raz następuje po dwóch piosenkach, a następnym
razem po trzech. Zakres od **0** pozwala krokowi pominąć niektóre kolejki:
przy złotych przebojach 0–1 złoty przebój raz gra po sygnale stacji, a raz nie.
Maksimum to zawsze co najmniej 1. Jedna rotacja jest **domyślna**: nowa
instalacja ma jedną, **Muzyka**, która gra całą Twoją muzykę.

Krok, którego kategoria nie ma żadnego utworu, który może zagrać (pusta
kategoria albo brakujące pliki), gra w swojej kolejce utwór z domyślnej
rotacji (dowolną muzykę, jeśli krok należy do samej domyślnej rotacji), więc
pozostałe kroki zachowują swoje miejsce: sygnał stacji nadal pojawia się
między piosenkami.

Rotacje tworzy się w Ustawienia → Automatyzacja:

![Ustawienia, Automatyzacja: rotacje Music (domyślna) i Hits, zaznaczona Music z jednym krokiem obejmującym całą muzykę, jej liczby Co najmniej i Co najwyżej oraz początek sekcji Szablony i dni](../../images/guide/settings-automation.png)

## Jak wybiera {#how-it-chooses}

W obrębie kroku wypełniacz muzyczny preferuje utwory, które najdłużej nie
były na antenie, z niewielką losowością między nimi, aby kolejność nigdy nie
stała się stałym cyklem. Pilnuje też odstępu:

- **ten sam utwór:** może zagrać ponownie, gdy minęły 3 godziny albo od tego
  czasu zagrała połowa pozostałych utworów jego kroku, zależnie od tego, co
  nastąpi pierwsze;
- **ten sam wykonawca:** nie w ciągu 30 minut.

Liczy się wszystko, co poszło na antenę: z wypełniacza muzycznego,
odtwarzacza lub cartwalla; odsłuch wstępny się nie liczy. Gdy mała kategoria
nie może dotrzymać reguł, wypełniacz muzyczny ustępuje, zamiast zamilknąć:
najpierw co do wykonawcy, potem co do utworu. Ustawienia → Automatyzacja
zawiera każdą wartość i tam tworzy się, zmienia nazwy i zmienia rotacje (zob.
[Ustawienia](settings.md#automation)).

Przed blokiem ramówki wypełniacz muzyczny wybiera też swoje ostatnie utwory
według długości, aby muzyka kończyła się czysto: utwór wyciszany na początku
bloku gra jeszcze o godzinie bloku i traci po niej najwyżej 30 sekund, a nie
zaczyna się żaden utwór, który grałby krócej niż 45 sekund. Reguł wykonawcy
i utworu też przestrzega, kiedy tylko może. Gdy żaden utwór rotacji nie może
spełnić obu warunków, gra ten, który traci najmniej, więc czasem może być
słychać kilka sekund utworu. Pauza muzyki przesuwa następujące po niej
utwory, a wypełniacz muzyczny wybiera je na nowo. Wypełniacz muzyczny unika
też utworów i wykonawców, które blok gra wkrótce potem. Obie wartości można
na razie zmienić tylko w pliku konfiguracyjnym
(`automation.max_fade_cut_secs` i `automation.min_song_play_secs`).

**Sygnał godzinowy** to osobny rodzaj utworu (Biblioteka → Rodzaj ▸ Sygnał
godzinowy) do identyfikacji stacji o pełnej godzinie.
