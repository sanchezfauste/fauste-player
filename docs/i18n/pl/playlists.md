# Playlisty

## Karty {#tabs}

Każdy odtwarzacz ma rząd kart, po jednej na playlistę. Wszystkie
odtwarzacze widzą te same playlisty; każdy wybiera, którą pokazuje. Kropka
na karcie pokazuje, gdzie są utwory odtwarzacza: **czerwona** dla utworu na
antenie, **zielona** dla następnego.

Karty dzielą szerokość odtwarzacza. Nazwa, która się nie mieści, kończy się
na „…”; najedź kursorem na kartę, aby przeczytać ją w całości. Przy wielu
playlistach karty przestają się zwężać po osiągnięciu minimalnej szerokości,
na końcach rzędu pojawiają się strzałki, a kółko myszy nad kartami je
przewija. Karta, którą wybierzesz, i ta, którą pokazuje odtwarzacz, są
wprowadzane do widoku.

Przełączanie kart nigdy nie zmienia tego, co jest na antenie ani co jest
następne. Gdy utwór się kończy, odtwarzacz kontynuuje w playliście, która
ten utwór zawiera.

Playlisty tworzy się, zmienia im nazwy i usuwa w [Ustawieniach](settings.md).
Nie można usunąć ostatniej playlisty ani playlisty z utworem na antenie.

## Tabela utworów {#the-track-table}

![Playlista: odtworzone utwory przyciemnione, utwór na antenie na czerwono, następny utwór na zielono oraz stopka z pozostałym czasem](../../images/guide/playlist.png)

| Kolumna | Zawartość |
|---|---|
| `#` | Pozycja, uzupełniona zerami; w przypadku bieżącego i następnego utworu zastępuje ją ikona |
| Tytuł | Z tagów albo z nazwy pliku (`Artist - Title.mp3` jest dzielone). Ikony powtarzania i stopu po utworze stoją przed nim |
| Wykonawca | Z tagów; „Nieznany wykonawca”, gdy go nie ma |
| Album | Z tagów |
| Data | Data nagrania w takiej postaci, w jakiej zapisuje ją plik (`2019`, `2019-05` lub `2019-05-14`, z godziną, jeśli jest) |
| Gatunek | Z tagów |
| Czas | Długość odtwarzania, od cue-in do cue-out (cały plik, gdy **Używaj cue-in i cue-out** jest wyłączone) |
| Intro | Jak długo trwa intro, od miejsca, w którym utwór zaczyna grać, do jego markera intro; puste, gdy utwór nie ma markera intro |
| Nazwa pliku | Nazwa pliku z rozszerzeniem |

Nowa instalacja pokazuje `#`, Tytuł, Wykonawcę i Czas. Pozostałe kolumny są
opcjonalne; zob. **Wybór kolumn** niżej. Utwór bez wartości pokazuje pustą
komórkę, z wyjątkiem Wykonawcy, który pokazuje „Nieznany wykonawca”.

Kolumny wypełniają tabelę i zachowują proporcje przy zmianie rozmiaru okna;
kolumny tekstowe dostają najwięcej miejsca. Przeciągnij separatory
nagłówków, aby zmienić proporcje: kolumny na prawo od separatora podążają
za wskaźnikiem w każdej klatce (dzielą to, co zostało, proporcjonalnie do
swojej szerokości), kolumny na lewo od niego zostają, a szerokości są
zapisywane po puszczeniu. Żadna kolumna nie zwęża się poniżej swojego
minimum. Szerokości są pamiętane dla każdego odtwarzacza; kolumna, którą
pokażesz później, zaczyna od domyślnej szerokości, a pozostałe zachowują
swoje proporcje.

### Wybór kolumn {#choosing-the-columns}

Tytuł i Czas są zawsze widoczne. Każdą inną kolumnę można pokazać lub
ukryć, a każdą kolumnę, także te dwie, można przesunąć. Lista jest taka sama
dla każdego odtwarzacza i każdej playlisty i jest zapisana w `config.json`
jako `ui.table_columns`. Trzy sposoby jej zmiany:

- **Ustawienia → Playlisty → Kolumny tabeli:** zaznacz kolumny do
  pokazania; strzałki przesuwają pokazaną kolumnę w górę lub w dół (w
  tabelach czyta się je od lewej do prawej). **Domyślne kolumny** wracają do
  `#`, Tytułu, Wykonawcy i Czasu.
- **Kliknij prawym przyciskiem nagłówek:** menu z polem wyboru dla każdej
  opcjonalnej kolumny. Kolumna, którą pokażesz, pojawia się na prawym końcu;
  przeciągnij ją stamtąd.
- **Przeciągnij nagłówek** na inny: upuść go na lewej połowie nagłówka, aby
  umieścić kolumnę przed nim, na prawej połowie, aby umieścić ją za nim.
  Upuszczenie gdziekolwiek indziej nic nie robi.

Nazwa w `ui.table_columns`, której ta wersja nie zna, jest ignorowana, a
brakujący Tytuł lub Czas jest dodawany z powrotem.

Po otwarciu aplikacji każda tabela przewija się tak, aby następny utwór jej
odtwarzacza znalazł się pośrodku tabeli (na ile pozwalają końce listy). Dzieje
się tak raz, przy uruchomieniu, i tylko gdy następny utwór jest w playliście
pokazywanej przez tabelę.

Gdy odtwarzacz przechodzi do innego utworu, jego tabela pokazuje playlistę
tego utworu i przewija jego wiersz na górę, chyba że korzystałeś z tabeli w
ciągu ostatnich 10 sekund (przewijałeś ją, przeciągałeś utwór, otwierałeś
menu utworu lub klikałeś kartę): wtedy czeka, aż zostawisz ją w spokoju na
tyle długo. Czas ten to `ui.follow_current_grace_secs` w `config.json`; 0
wyłącza podążanie.

Odtwarzacze są niezależne: kilka odtwarzaczy może pokazywać tę samą
playlistę, każdy z własnym następnym utworem, własnymi znacznikami
odtworzenia i własnymi czasami w stopce. Odtwarzanie, zatrzymywanie lub
pomijanie w jednym odtwarzaczu nigdy nie przesuwa następnego utworu innego
odtwarzacza. Ten sam utwór może nawet być na antenie w dwóch odtwarzaczach
jednocześnie. Edycja playlisty (dodawanie, przenoszenie lub usuwanie
pozycji) albo plik, który staje się nieczytelny, może jednak zmienić
następny utwór każdego odtwarzacza, który ją pokazuje.

Kolory wierszy:

| Wiersz | Znaczenie |
|---|---|
| **Czerwony**, z ikoną głośnika (lub pauzy) | Na antenie w tym odtwarzaczu. Może też pokazywać zieloną strzałkę: utwór na antenie jest także następnym, więc zagra jeszcze raz |
| Czerwone **P2** (lub inny numer) w kolumnie numeru | Na antenie w tamtym odtwarzaczu |
| **Zielony**, ze strzałką | Następny utwór tego odtwarzacza |
| Przyciemniony | Już odtworzony w tym odtwarzaczu |
| Plik z krzyżykiem / ikona ostrzeżenia | Plik brakuje / jest nieczytelny (jest pomijany); najedź na wiersz: okienko zaczyna się od przyczyny, na bursztynowo, a potem zwykłe pola. Brakującego pliku szuka się ponownie co 30 s (`tuning.missing_recheck_ms`). |
| Strzałki odświeżania po prawej stronie tytułu | Przeanalizowany przez wcześniejszą wersję; nadal gra z tą analizą. **Ustawienia → Analiza → Analizuj nieaktualne utwory** aktualizuje go (utwory w odtwarzaczu i tak są aktualizowane) |
| Klepsydra po prawej stronie tytułu | Utwór czeka na analizę (najedź na klepsydrę: *Analiza oczekuje*). Mimo to gra, a klepsydra znika po zakończeniu analizy |
| Fioletowy | Zaznaczony |

**Podpowiedź utworu.** Najedź na chwilę na wiersz, aby zobaczyć tytuł, wykonawcę, album, datę, gatunek, długość, format (typ, częstotliwość próbkowania i głębię bitową, gdy są znane) oraz ścieżkę pliku. Pole, którego plik nie ma, jest pomijane. Okienko to jedyna informacja po najechaniu na wiersz i nigdy się nie przesuwa, gdy jest pokazane. Dla brakującego lub nieczytelnego pliku zaczyna się od przyczyny.

## Mysz {#mouse}

- **Kliknięcie** zaznacza utwór. **Dwuklik** ustawia go jako następny w tym
  odtwarzaczu. Na utworze na antenie sprawia, że zagra jeszcze raz po
  zakończeniu bieżącego przebiegu.
- **Kliknięcie prawym przyciskiem** otwiera menu kontekstowe:

![Menu kontekstowe utworu](../../images/guide/track-menu.png)

| Pozycja | Działanie |
|---|---|
| Odtwórz teraz | Uruchamia ten utwór od razu (z miksem, jeśli odtwarzacz jest na antenie) |
| Ustaw jako następny | To samo co dwuklik. Na utworze na antenie zagra jeszcze raz, od początku, po zakończeniu bieżącego przebiegu (miksując jak Powtarzaj, bez przerwy), po czym odtwarzacz idzie dalej. Działa jednorazowo. Stop po, tryb SINGLE i oznaczenie Stop po nadal najpierw kończą pracę odtwarzacza. Gdy działa CUE, przechodzi na nowy następny utwór |
| Odsłuchaj na CUE | Odtwarza go na wyjściu CUE (otwiera okno CUE). Przyciemnione, gdy odtwarzacz nie ma wyjścia Cue innego niż Main |
| Edytuj tagi… | Otwiera edytor tagów dla tego utworu. **Zapisz** zapisuje zmiany w pliku audio; **Anuluj** (lub Esc, gdy nie trwa zapis) zamyka bez zapisywania. Pozycja jest przyciemniona, z przyczyną po najechaniu, gdy utwór jest na antenie, na CUE lub w grającym carcie, gdy jego tagi nie zostały jeszcze odczytane, gdy brakuje pliku oraz dla formatów, których tagów nie da się zapisać (na przykład DSD) |
| Analizuj ponownie | Analizuje ten utwór od nowa, bez względu na jego stan. Naprawiony plik, który był nieczytelny, jest też podejmowany samoczynnie (zob. [Rozwiązywanie problemów](troubleshooting.md)). Markery ręczne są zachowywane |
| Dodaj utwory poniżej… | Wybierz pliki do wstawienia po tym utworze |
| Duplikuj | Wstawia poniżej nieodtworzoną kopię (z jej oznaczeniami powtarzania i stopu po) |
| Powtarzaj ten utwór | Zaznacz, aby grać go w kółko, bez przerwy, dopóki nie naciśniesz Play (następny), Poprzedni, Stop lub Stop z wyciszeniem albo nie włączysz Stop po. Pauza zachowuje powtarzanie. Ikona powtarzania pojawia się przed tytułem |
| Stop po tym utworze | Zaznacz, aby zatrzymać odtwarzacz po zakończeniu tego utworu, za każdym razem, gdy jest grany (w dowolnym trybie). W odróżnieniu od przycisku **Stop po** odtwarzacza oznaczenie zostaje przy utworze i jest zapisywane z playlistą. Ikona stopu po pojawia się przed tytułem. Ma pierwszeństwo przed Powtarzaj |
| Przenieś do ▸ | Przenosi go na koniec innej playlisty |
| Usuń z playlisty | Usuwa go; niemożliwe, gdy jest na antenie |

## Edycja tagów {#editing-tags}

**Edytuj tagi…** otwiera okno dla jednego utworu. Dopóki jest otwarte, żaden
skrót klawiszowy nie działa, a pliki upuszczone na okno aplikacji są
ignorowane.

![Edytor tagów pliku FLAC, z okładką, tytułem, wykonawcą, albumem, datą i gatunkiem](../../images/guide/tag-editor.png)

- **Co widać.** Edytor odczytuje plik przy otwarciu (w międzyczasie pokazuje
  „Odczytywanie tagów…”). Zawsze widoczne: tytuł, wykonawca, album,
  wykonawca albumu, data, numer utworu i suma, numer płyty i suma, gatunek,
  kompozytor i komentarz. Widoczne, gdy plik je ma: podtytuł, grupowanie,
  BPM, tonacja początkowa, nastrój, ISRC, wydawca, numer katalogowy, prawa
  autorskie, oryginalny wykonawca, oryginalny album, oryginalna data
  wydania, autor tekstu, dyrygent, remikser, aranżer, wykonawca utworu,
  język, zakodowane przez, tekst utworu, tytuł do sortowania, wykonawca do
  sortowania, album do sortowania, wykonawca albumu do sortowania,
  kompozytor do sortowania i strona wykonawcy.
- **Dodaj pole.** Menu pod polami wymienia pozostałe pola. Oferuje tylko to,
  co potrafi przechować format tagów pliku (WAV z RIFF INFO, AIFF czy stary
  tag ID3v1 przechowują mniej pól niż ID3v2, FLAC lub MP4) i jest
  przyciemnione, gdy nie ma już nic do dodania. Jedno z zawsze widocznych
  pól, którego format nie potrafi przechować, jest wyszarzone z uwagą.
  Wyczyszczenie pola usuwa je z pliku; dodane pole pozostawione puste nie
  jest zapisywane.
- **Kilka wartości.** Pola, które mogą mieć kilka wartości (wykonawca,
  wykonawca albumu, gatunek, kompozytor, nastrój i wpisy twórców, takie jak
  autor tekstu, dyrygent, remikser, aranżer i wykonawca utworu, oraz
  język), pokazują jedną wartość w wierszu; **Zapisz** zapisuje jedną
  wartość w wierszu w sposób właściwy dla formatu. Komentarz i tekst utworu
  to dowolny tekst w wielu wierszach.
- **Sprawdzanie.** Data i oryginalna data wydania mają format ISO 8601
  (`2019`, `2019-05` lub `2019-05-14`, opcjonalnie z godziną); numer utworu
  i płyty, ich sumy oraz BPM to liczby całkowite, a suma wymaga numeru. Pole
  z nieprawidłową wartością jest oznaczone, a **Zapisz** pozostaje
  wyłączone. Wartość, którą plik już miał, a której nie ruszałeś, jest
  zachowywana bez zmian.
- **Pola zbyt długie.** Pole, którego tekst jest dłuższy niż
  `limits.max_tag_chars` albo które ma więcej wartości niż
  `limits.max_tag_values`, jest pokazane tylko do odczytu z uwagą „Za długie,
  by edytować tutaj; pozostaje bez zmian w pliku”. Nigdy nie jest zapisywane
  z powrotem, więc zapis nie może go przyciąć.
- **Co jest zachowywane.** Wszystko, czego edytor nie pokazuje (inne
  standardowe klucze, klucze własne, obrazy inne niż okładka przednia,
  ramki binarne), zostaje w pliku z tymi samymi wartościami. Edytor podaje,
  ile takich tagów zachowuje (i „kolejne”, gdy format ma ramki, których nie
  da się policzyć). Zapis koduje na nowo pozycje, które edytor mapuje, więc
  zachowana pozycja może się różnić bajtami (kodowaniem tekstu, kolejnością
  ramek), ale nie wartością.
- **Okładka.** Edytor pokazuje okładkę przednią albo pierwszy obraz w
  pliku, gdy nie ma okładki przedniej, jako miniaturę.
  - **Zmień…** otwiera okno wyboru pliku dla obrazu JPEG lub PNG (co
    najwyżej `limits.max_cover_bytes`, i musi dać się zdekodować). Jeśli się
    nie da, edytor mówi dlaczego i nic się nie zmienia.
  - **Usuń** czyści okładkę przednią. Jest wyłączone, gdy plik nie ma
    okładki przedniej: obraz pokazany tylko dlatego, że nie ma okładki
    przedniej, służy wyłącznie do wyświetlania i jest zachowywany bez zmian.
  - Okładka, która jest w pliku, ale nie można jej pokazać (obraz, który się
    nie dekoduje, albo GIF, BMP lub WebP), jest zapowiadana komunikatem „Nie
    można wyświetlić tej okładki; zostaje zachowana bez zmian”. **Zmień…** i
    **Usuń** nadal działają.
  - Zmianę zapisuje **Zapisz**, a odrzuca **Anuluj**. Okładek tylnych i
    żadnych innych obrazów nigdy się nie rusza. Format bez miejsca na obrazy
    (WAV z RIFF INFO, AIFF, ID3v1) pokazuje ten obszar jako wyłączony. Po
    zapisie okładka odtwarzacza pokazuje nową okładkę.
- **Jak działa zapis.** Plik jest kopiowany obok oryginału, kopia dostaje
  tagi, jest synchronizowana i zastępuje oryginał, więc awaria zostawia plik
  takim, jaki był. Przyczyna pojawia się w edytorze, który pozostaje
  otwarty, by spróbować ponownie, oraz na pasku stanu. Zapisywane są tylko
  pola, które zmieniłeś. Po zapisie tabela od razu pokazuje nowe tagi, a
  markery i przebieg są zachowywane. Jeśli plik nie zachował zmienionego
  przez Ciebie pola, pasek stanu je wymienia.
- **Po aktualizacji.** Utwory z wcześniejszej wersji mają datę, gatunek i
  inne tagi uzupełniane po cichu w tle (bez pełnej analizy).

## Przeciąganie i upuszczanie {#drag-and-drop}

- Przeciągnij utwór w obrębie listy, aby zmienić kolejność. Fioletowa linia
  pokazuje, gdzie wyląduje: jest na granicy wierszy najbliższej wskaźnikowi
  i tylko na liście pod wskaźnikiem. Puszczenie nad nagłówkiem, krawędzią
  kolumny, paskiem przewijania albo oknem zasłaniającym listę (oknem CUE)
  niczego nie upuszcza.
- Przeciągając utwór, przytrzymaj wskaźnik przy górnej lub dolnej krawędzi
  listy, aby ją przewijać: im bliżej krawędzi, tym szybciej, a przewijanie
  kończy się na końcach listy lub gdy oddalisz się od krawędzi. Kółko myszy
  też przewija listę podczas przeciągania. Fioletowa linia nadal podąża za
  wskaźnikiem, gdy lista się przesuwa. Przeciąganie plików z menedżera
  plików nad listą przewija ją w ten sam sposób tam, gdzie system podaje
  pozycję wskaźnika, gdy tylko przesuniesz wskaźnik nad listę.
- Przeciągnij go na listę innego odtwarzacza, aby go tam przenieść.
- Przeciągnij go na kartę, aby dołączyć go do tej playlisty.
- Upuść pliki lub foldery z menedżera plików na listę, aby wstawić je w
  miejscu upuszczenia. Nad nagłówkiem, krawędzią kolumny, paskiem przewijania
  lub oknem zasłaniającym listę nic nie jest wstawiane. Jeśli system nie
  podaje pozycji, trafiają na koniec pokazywanej listy.

## Stopka {#footer}

**+ Dodaj** otwiera okno wyboru plików, zaczynając w folderze z muzyką
ustawionym w Ustawieniach. **Zeruj odtworzone** (ikona strzałki obok)
czyści przyciemniony znacznik „już odtworzony” każdego utworu playlisty,
dla każdego odtwarzacza, po zapytaniu „Wyczyścić znacznik odtworzenia
każdego utworu na tej playliście?” (**Anuluj**, Esc lub kliknięcie poza
oknem zachowują znaczniki). Utwór na antenie zachowuje swój stan i jest
oznaczany, gdy odtwarzacz go opuści. Przycisk jest przyciemniony, gdy nie
ma nic do wyczyszczenia. Stopka pokazuje też liczbę utworów, czas
pozostały w playliście i jej całkowitą długość.

## Pliki playlist {#playlist-files}

- **Import:** Ustawienia → Playlisty → **Importuj M3U / PLS…** albo upuść
  plik `.m3u`, `.m3u8` lub `.pls` na okno. Staje się nową playlistą o nazwie
  pliku.
  - Ścieżki względne są rozwiązywane względem folderu pliku playlisty.
  - Adresy `file://` są rozumiane.
  - Pliki, których nie można znaleźć, są mimo to dodawane, oznaczone jako
    niedostępne.
  - Strumienie internetowe są pomijane; komunikat podaje, ile ich było.
- **Eksport:** przycisk **M3U** przy każdej playliście w Ustawieniach zapisuje
  ją jako plik M3U8 z tytułami, długościami i pełnymi ścieżkami.
