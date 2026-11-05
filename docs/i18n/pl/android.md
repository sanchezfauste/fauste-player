# Android

Fauste Player działa też na telefonach i tabletach z Androidem (Android 10 lub
nowszy), na tym samym rdzeniu emisji co wersja na komputer: odtwarzacze,
playlisty, cartwall, markery, normalizacja głośności i każda reguła
odtwarzania działają tak samo.

## Instalacja {#install}

Wydanie, które zawiera aplikację, ma na stronie **Releases** plik
`fauste-player-<version>-android.apk` (z jego `.sha256`). Otwórz go na
telefonie i zezwól przeglądarce lub menedżerowi plików na instalowanie
aplikacji, gdy Android o to zapyta. Aby zaktualizować, zainstaluj nowszy APK w
ten sam sposób: Twoje playlisty i ustawienia zostają. Aplikacji nie ma jeszcze
w Google Play.

## Pierwsze uruchomienie {#first-start}

- **Dostęp do muzyki.** Zezwól na dostęp do muzyki (oraz do powiadomień, w
  Androidzie 13 i nowszym). Bez tego przeglądarka muzyki pozostaje pusta.
- **Emisja trwa dalej.** Gdy odtwarzacz lub cart gra, powiadomienie pozostaje
  widoczne, a dźwięk trwa przy wyłączonym ekranie lub aplikacji w tle. Tak jak
  na komputerze, nic nie trafia na antenę samo po starcie ani awarii.
- **Wstecz i zamykanie.** Wstecz najpierw zamyka to, co jest otwarte
  (przeglądarkę muzyki, Ustawienia, O programie, otwarty odtwarzacz, stronę
  karty Biblioteka). Gdy nie ma już nic do zamknięcia, Wstecz chowa aplikację,
  póki coś brzmi (odtwarzacz, cart lub odsłuch wstępny CUE), i gra dalej; gdy
  nic nie brzmi, zapisuje i zamyka aplikację. Zmiecenie aplikacji w
  przełączniku aplikacji robi to samo: zmiecenie nigdy nie zatrzymuje tego, co
  gra, a aplikacja zamyka się, gdy ucichnie.
- **Dźwięk innych aplikacji.** Gdy inna aplikacja przejmuje dźwięk, Fauste
  Player gra dalej i mówi o tym na pasku stanu u dołu.

## Ekran {#the-screen}

Na **telefonie** górny pasek ma cztery karty, **Na żywo**, **Cartwall**,
**Playlisty** i **Biblioteka**, pokazane jako ikony (wybrana także z nazwą,
gdy jest miejsce), a potem **kłódkę** i **⋮** (Więcej: Ustawienia i O
programie). Zegar pokazuje się, gdy jest dla niego miejsce. Po obróceniu na
bok ekran Playlisty umieszcza swoje żetony odtwarzaczy (P1, P2…) na górnym
pasku, a pasek stanu u dołu pokazuje się tylko wtedy, gdy ma coś do
powiedzenia.

<p>
<img src="../../images/android/1-live.png" width="180" alt="Na żywo na telefonie: dwa odtwarzacze jako karty, jedna cała i początek następnej, a pod nimi cartwall">
<img src="../../images/android/2-carts.png" width="180" alt="Carty na telefonie: strona General ustawiona pionowo, dwa carty w wierszu">
<img src="../../images/android/3-playlists.png" width="180" alt="Playlisty na telefonie: żetony P1 do P4, karty playlist, lista utworów i karta odtwarzacza u dołu">
<img src="../../images/android/4-music.png" width="180" alt="Muzyka na urządzeniu na telefonie: folder Demo z trzema zaznaczonymi utworami i pasek do dodania ich do playlisty Morning odtwarzacza P1">
</p>

- *Na żywo:* każdy odtwarzacz jako karta oraz cartwall. Każda karta ma, od
  góry do dołu, nazwę odtwarzacza, stan, Single/Cont i CUE; okładkę, tytuł i
  wykonawcę oraz następny utwór; przebieg (stuknięcie przeskakuje); potem czas,
  który upłynął, i długość utworu z czasem pozostałym dużymi cyframi nad
  transportem (Play/Następny, Poprzedni, Pauza, Stop, Od początku, Stop z
  wyciszeniem, Stop po bieżącym) z przyciskami wielkości palca oraz smukły
  miernik z suwakiem głośności (poziom suwaka pokazuje się pod nim). Czas
  pozostały kończy się tam, gdzie kończą się przyciski, mniejszy przy długim
  utworze. Na większości telefonów miernik pokazuje swoją skalę po lewej, a na
  szerszym (około 406 dp i więcej) suwak pokazuje też swoje poziomy; na wąskim
  telefonie (poniżej około 384 dp) żadne z nich, a przyciski są trochę
  mniejsze, by się zmieściły. Przyciski działają, gdy inny palec trzyma suwak, a
  stuknięcie się liczy, choć palec się trochę przesunie lub chwilę zostanie.
  Trzymany pionowo ekran ma karty u góry, jedną całą i początek następnej
  (przewiń po więcej), a cartwall pod nimi. Obrócony na bok, każdy odtwarzacz
  jest jednym wierszem po lewej (jego nazwa i stan, tytuł, czas pozostały,
  Play/Następny, Pauza i Stop; stuknij wiersz gdzie indziej, aby otworzyć
  odtwarzacz w Playlistach), a cartwall jest po prawej. Stuknij **CARTWALL** w
  jego nagłówku, aby go zwinąć: zostaje tylko nagłówek, u dołu (karty stron i
  *Zatrzymaj wszystko* nadal działają), a odtwarzacze dostają miejsce; stuknij
  ponownie, aby rozwinąć. Karta Cartwall zawsze pokazuje cały cartwall. Strona
  cartów zbyt szeroka na dostępne miejsce pokazuje się w mniejszej liczbie
  kolumn, w tej samej kolejności.
- *Cartwall:* pokazywana strona cartów, wypełniająca ekran. Przesuń w bok, aby
  pokazać następną lub poprzednią stronę. Trzymana pionowo strona szersza niż
  wysoka jest obrócona (8 × 2 pokazuje się jako 2 × 8), w tej samej kolejności.
- *Playlisty:* żetony u góry (P1, P2…) wybierają odtwarzacz; pod nimi są jego
  karty playlist (+ tworzy nową), lista utworów i, u dołu (po prawej, gdy
  telefon leży na boku), karta odtwarzacza z większym przebiegiem do
  przygotowania utworu (staje się niższa, gdy brakuje miejsca, więc przyciski
  zawsze są widoczne). Długie przytrzymanie wiersza otwiera jego menu, które ma
  też **Przenieś w górę**, **Przenieś w dół** i **Przenieś do innego
  odtwarzacza** (utwór trafia zaraz po następnym utworze tego odtwarzacza, w
  playliście, którą pokazuje; utwór na antenie zostaje). Przeciągnij wiersz za
  uchwyt ≡, aby go przenieść. Stopka ma **Dodaj** i, za **⋯**, **Zeruj
  odtworzone**. Długie przytrzymanie przebiegu ustawia w tym miejscu marker
  (cue-in, koniec intro, początek outro, MIX, cue-out) lub usuwa ustawiony
  przez Ciebie; markery edytuje się tylko tutaj, nie na Na żywo.
- *Biblioteka:* biblioteka utworów i muzyka urządzenia (zob.
  [Biblioteka](#library)).

Wstecz z innego ekranu prowadzi do Na żywo; w Bibliotece najpierw wraca do
listy kategorii (w Muzyce na urządzeniu najpierw o folder wyżej).

Gdy nic na ekranie się nie rusza, aplikacja rysuje go raz na sekundę (na
potrzeby zegara), aby oszczędzać baterię; miernik, który wciąż opada po pauzie
lub zatrzymaniu, utrzymuje rysowanie, aż opadnie.

**Kłódka** chroni przed przypadkowymi dotknięciami podczas emisji. Zablokowane
(kłódka zamknięta i podświetlona), suwaki głośności tylko pokazują poziom, a
przebiegi ani nie przeskakują, ani nie edytują markerów; wyglądają na
przyciemnione. Szczypanie i przeciąganie nadal powiększa i przesuwa przebieg, a
Play, transport, carty i playlisty zawsze działają. Blokada zostaje po
ponownym uruchomieniu i jest zapisywana z sesją, więc wczytanie innej sesji
przejmuje blokadę tej sesji; zdalne API i MIDI nie są blokowane.

Przeciągnięcie, które zaczyna się na suwaku głośności, go przesuwa; to, które
zaczyna się gdziekolwiek indziej, przewija, więc przewijanie nigdy nie zmienia
poziomu na antenie. Stuknięcie przebiegu przeskakuje tam, szczypnięcie dwoma
palcami powiększa wokół palców, przeciągnięcie jednym palcem przesuwa
powiększony widok, a **Pełny widok** znów pokazuje cały utwór. W playliście
palec przewija; aby przenieść utwór, użyj jego uchwytu ≡ lub menu wiersza.

Na **tablecie** układ jest układem z komputera (zob. [Układ](layout.md)), z
celami łatwiejszymi do trafienia.

Długie przytrzymanie otwiera to, co na komputerze otwiera kliknięcie prawym
przyciskiem: menu wiersza, menu nagłówków, menu carta (**Edytuj…** wybiera plik
pustego carta). Na telefonie Ustawienia wypełniają ekran i wymieniają swoje
sekcje; stuknij jedną, aby ją otworzyć, z nazwą każdego ustawienia nad jego
kontrolką. Strzałka lub Wstecz wraca do listy, a Wstecz jeszcze raz (albo
**Zamknij**) zamyka Ustawienia. Edycja carta otwiera jego sekcję bezpośrednio.

O programie → **Licencje stron trzecich** otwiera w przeglądarce lub
przeglądarce HTML informacje o bibliotekach, z których zbudowano aplikację.

## Biblioteka {#library}

Na **telefonie** karta **Biblioteka** otwiera się na liście kategorii:
**Wszystkie utwory**, Twoje kategorie (każda podkategoria pod swoją nadrzędną,
z kolorem i liczbą utworów), **Bez kategorii** i **Muzyka na urządzeniu**.
Biblioteka jest tą samą, którą pokazuje [okno biblioteki](library.md) na
komputerze: jest w niej każdy plik dodany do playlisty, carta lub biblioteki.
Kategorie tworzy się, zmienia im nazwy i kolory oraz usuwa w **Ustawienia →
Biblioteka**.

**Utwory kategorii.** Stuknij kategorię, aby otworzyć jej utwory (razem z
utworami jej podkategorii). Strzałka w jej nagłówku lub Wstecz wraca do
kategorii. Inna karta i powrót zastaje listę taką, jak ją zostawiłeś. Otwarcie
kategorii z listy kategorii zaczyna od pustego wyszukiwania. Wyszukiwanie
filtruje listę po tytule, wykonawcy, albumie i pozostałych tagach, które
przeszukuje okno biblioteki; **Cała biblioteka** przeszukuje każdą kategorię.
Każdy wiersz pokazuje tytuł (albo nazwę pliku), wykonawcę, czas trwania i
oznaczenia okna dla pliku brakującego, nieczytelnego lub jeszcze
nieprzeanalizowanego.

Stukaj wiersze, aby je zaznaczać. Gdy coś jest zaznaczone, u dołu pojawia się
pasek Muzyki na urządzeniu: ile, odtwarzacz i playlista, do których trafią
(stuknij, aby wybrać inne), **Dodaj na końcu** i **Dodaj po następnym**, tak
jak w Muzyce na urządzeniu. Utwory trafiają w kolejności zaznaczania, a
dodanie czyści zaznaczenia. Powrót do kategorii też je czyści.

**Menu wiersza.** Długie przytrzymanie wiersza otwiera jego menu: **Dodaj do ▸**
(playlista odtwarzacza, na końcu), **Odsłuchaj**, **Przenieś do kategorii ▸**,
**Analizuj ponownie** i **Usuń z biblioteki…**, co pyta tak jak na komputerze
(zob. [Usuwanie utworów](library.md#removing-tracks)). Gdy wiersze są
zaznaczone, długie przytrzymanie innego wiersza zaznacza go także, a menu
działa na każdy zaznaczony wiersz, w kolejności zaznaczania; **Odsłuchaj** nie
jest wtedy oferowane. Zaznaczenia zostają po akcji z menu.

**Odsłuch wstępny.** Słuchawki 🎧 po prawej stronie wiersza odtwarzają jego
utwór od cue-in na wyjściu odsłuchu wstępnego biblioteki; 🎧 innego wiersza
przełącza na ten utwór. Smukły pasek nad listą pokazuje tytuł, czas
pozostały, **Wstrzymaj odsłuch**/**Odsłuch** i **Zatrzymaj odsłuch**, gdy
gra (lub jest wstrzymany), a w przeciwnym razie podpowiedź: zawsze zachowuje
swoje miejsce, więc wiersze nie przesuwają się, gdy odsłuch zaczyna się lub
kończy. 🎧 jest przyciemnione, gdy plik jest brakujący lub nie można go
odtworzyć. Podgląd zatrzymuje się, gdy jego utwór opuszcza bibliotekę i gdy
opuścisz kartę Biblioteka (na inną kartę lub przez Wstecz); na liście
kategorii lub w Muzyce na urządzeniu gra dalej, ale jego pasek pokazuje się
tylko na liście utworów.

W odróżnieniu od komputera Android (telefon lub tablet) odtwarza odsłuch
wstępny biblioteki na wyjściu domyślnym, gdy nie wybrano dla niego wyjścia: w
**Ustawienia → Wyjścia audio → Biblioteka** pusty wybór brzmi *Domyślne
systemowe*, a jego **Test Cue** gra tam. Wybierz tam inne wyjście (na przykład
słuchawki przewodowe), aby słuchać wstępnie z dala od anteny. Wyjście domyślne
jest zwykle także wyjściem odtwarzaczy i cartwalla, więc odsłuch wstępny może
być słyszalny na antenie: gdy coś gra na tym wyjściu, 🎧 najpierw pyta
(**Odsłuchaj mimo to** albo **Anuluj**), a odsłuch wstępny, którego nie
potwierdziłeś, zatrzymuje się, gdy coś tam startuje, tak jak CUE odtwarzacza
(zob. [CUE (odsłuch wstępny)](players.md#cue-pre-listen)).

**Dodawanie muzyki urządzenia do kategorii.** **＋** w nagłówku listy utworów
otwiera Muzykę na urządzeniu dla tej kategorii. Jej pasek ma wtedy tylko
**Dodaj do &lt;kategoria&gt;** (**Dodaj do „Bez kategorii”** z *Wszystkie utwory* lub
*Bez kategorii*), co dodaje zaznaczone pliki do biblioteki w tej kategorii i
wraca do listy; plik, który już jest w bibliotece, dołącza do niej, jeśli nie
ma kategorii, a w przeciwnym razie zachowuje tę, którą ma, jak na komputerze.
Muzyka na urządzeniu zachowuje zaznaczenia między odwiedzinami, więc pliki
zaznaczone podczas wcześniejszych odwiedzin też są dodawane: najpierw sprawdź
liczbę na pasku. **Wróć do listy** u góry albo Wstecz wraca do listy bez
dodawania. Muzyka na urządzeniu otwarta z listy kategorii ma u góry zamiast
tego **Wróć do kategorii**.

**Folder urządzenia dla kategorii.** Kategoria może brać swoje pliki z folderu
muzyki urządzenia. W jej menu (długie przytrzymanie w **Ustawienia →
Biblioteka** albo w oknie biblioteki na tablecie) **Użyj folderu
urządzenia…** otwiera Muzykę na urządzeniu, aby wybrać folder (**Użyj tego
folderu**); menu kategorii podaje wtedy jego nazwę, nad swoimi pozycjami
folderu. Nic nie obserwuje folderu: gdy skopiujesz tam nową muzykę, stuknij
**↻** w nagłówku listy utworów kategorii (albo **Wczytaj ponownie folder
urządzenia** w jej menu). Każdy plik, który Muzyka na urządzeniu wymienia w
folderze i jego podfolderach, jest dodawany do kategorii, według powyższych
reguł (plik już w innej kategorii ją zachowuje), a komunikat mówi, ile plików
wczytano. Pliki, które opuściły folder, zostają w bibliotece; **Analizuj
ponownie** oznacza te, których plik zniknął, jako brakujące. **Zmień folder
urządzenia…** i **Zapomnij folder urządzenia** są w tym samym menu; jeden
folder należy do jednej kategorii.

### Muzyka na urządzeniu {#device-music}

**Muzyka na urządzeniu** pokazuje pliki audio urządzenia według **Foldery**
(ze ścieżką nad listą; stuknij jej część, aby wrócić wyżej), **Wykonawcy** lub
**Albumy** oraz wyszukiwanie po tytule, wykonawcy, albumie i nazwie pliku.
Każdy wiersz pokazuje tytuł, wykonawcę i czas trwania; stukaj wiersze, aby je
zaznaczać. Gdy coś jest zaznaczone, pasek u dołu pokazuje ile, odtwarzacz i
playlistę, do których trafią (stuknij, aby wybrać inne), **Dodaj na końcu** i,
gdy następny utwór tego odtwarzacza jest w tej playliście, **Dodaj po
następnym**. Utwory trafiają w kolejności zaznaczania. Plik urządzenia nie ma
odsłuchu wstępnego.

**Dodaj** w stopce playlisty, plik carta i Ustawienia → Playlisty → Folder z
muzyką otwierają Muzykę na urządzeniu na telefonie (na tablecie okno
**Muzyka na tym urządzeniu**, według folderu). Zaznacz pliki i stuknij
**Dodaj**, stuknij jeden plik dla carta albo **Użyj tego folderu**; **Anuluj**
lub Wstecz wraca tam, gdzie byłeś. Skopiuj muzykę na urządzenie (na przykład do
jego folderu *Music*), aby ją tam zobaczyć. Fauste Player tylko czyta Twoją
muzykę: tagi i okładki nie są zapisywane, a playlisty i strony cartów nie są
jeszcze importowane ani eksportowane w Androidzie.

Na **tablecie** biblioteką jest [okno biblioteki](library.md) z komputera,
otwierane z górnego paska.

## Wyjścia {#outputs}

Ustawienia → Wyjścia audio wymieniają wyjścia telefonu według rodzaju:
głośnik, słuchawki przewodowe, interfejs USB, głośnik Bluetooth („JBL Flip
(Bluetooth)”). Każdy odtwarzacz, cartwall i ich CUE wybierają jedno, tak jak
na komputerze, podobnie jak odsłuch wstępny biblioteki (zob.
[Biblioteka](#library)). Głośnik Bluetooth wyłączony i włączony ponownie jest
znajdowany znów po swojej trasie; gdy go nie ma, jego odtwarzacze zachowują się
jak przy brakującym urządzeniu na komputerze. To, czy dwa wyjścia mogą brzmieć
w tym samym czasie, zależy od telefonu.

## Czego nie ma w Androidzie {#not-on-android}

Powierzchnie sterujące MIDI, zdalne sterowanie, edycja tagów i okładek oraz
pliki playlist i stron cartów to na razie funkcje komputera. Klawiatura
podłączona do tabletu działa jak na komputerze (zob. [Klawiatura](keyboard.md)).
