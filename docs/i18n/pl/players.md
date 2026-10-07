# Odtwarzacze

Każda kolumna to jeden odtwarzacz. Odtwarzacze są niezależne: każdy ma własne
karty playlist, transport, głośność i wyjścia.

![Odtwarzacz 1 na antenie: nagłówek, okładka, tytuł, następny utwór, transport, odliczanie, miernik, suwak i przebieg](../../images/guide/player.png)

## Nagłówek {#header}

| Element | Znaczenie |
|---|---|
| `P1` … `Pn` | Numer odtwarzacza (klawisz numeryczny, który go uruchamia) |
| Kropka i etykieta stanu | **Na antenie** (czerwona), **Wstrzymany** (bursztynowa), **Zatrzymany** (szara) |
| Plakietka **Miks** / **Wyciszanie** | Trwa przenikanie do następnego utworu albo zatrzymanie z wyciszeniem |
| Plakietka **Stop po** | Odtwarzacz zatrzyma się, gdy skończy się bieżący utwór |
| Plakietka **Powtarzaj** / **Stop po utworze** | Bieżący utwór się powtarza albo zatrzymuje odtwarzacz po zakończeniu, z powodu własnego oznaczenia w menu playlisty. Najedź kursorem, aby zobaczyć pełne zdanie. Własny przycisk **Stop po** odtwarzacza ma pierwszeństwo: gdy jest włączony, widać tylko jego plakietkę |
| **BP** / **DSD** | **BP** świeci, gdy bieżący utwór dociera do urządzenia Main bez zmian. **DSD** zastępuje go, gdy utwór DSD wychodzi jako DSD, bez zmian (zob. [Wyjście bit-perfect](bit-perfect.md)). Obok pojawia się **Inne wyciszone**, gdy ten strumień DSD trzyma inne źródła z dala od wyjścia |
| **SINGLE** \| **CONT** | Tryb odtwarzania (zob. niżej): jeden połączony przełącznik, podświetlona połowa to aktywny tryb |
| **CUE** | Odsłuch wstępny następnego utworu na wyjściu CUE |

## Wiersz informacji {#info-row}

- **Okładka** utworu na antenie albo zastępcza płyta winylowa.
- **Tytuł i wykonawca** utworu na antenie. Zatrzymany odtwarzacz pokazuje
  utwór, który uruchomi Play (jego następny), z okładką, długością i
  przebiegiem, gotowy na cue-in albo w miejscu, w które kliknięto jego
  przebieg.
- Utwór odtwarzany lub wczytany przed końcem analizy ma już swoją długość,
  jeśli podaje ją nagłówek pliku: odliczanie, `upłynęło / całość` i
  przeskok kliknięciem działają od razu, nad płaską linią, dopóki przebieg
  nie jest gotowy. Gdy nagłówek jej nie przechowuje (surowe pliki AAC, pliki
  MP3 bez ramki długości, Matroska i WebM), całość pokazuje „—”, a w przebieg
  nie można klikać, dopóki analiza się nie skończy.
- **Miernik stereo** (w kolumnie po prawej stronie odtwarzacza, obok suwaka;
  obejmuje wiersz informacji i transport): poziom, który odtwarzacz wysyła,
  po uwzględnieniu jego głośności.
  - Jeden ciągły słupek na kanał, w skali określonej normą danego typu
    miernika (domyślnie miernik szczytu cyfrowego: od −60 do 0 dBFS, przy
    czym górne 20 dB zajmuje połowę wysokości). Skala to podziałka po
    każdej stronie słupków, opisana w jednostkach samego miernika po obu
    stronach. Jej góra i dół (w mierniku cyfrowym dół skali) są zawsze
    oznaczone; każda etykieta ma kreskę na każdej podziałce, a krótsze
    kreski między etykietami działają jak na liniale: rozmieszczone równo
    na okrągłych wartościach, co 1 dB w mierniku EBU i co 5 dB poniżej −20
    w cyfrowym, gdy miernik jest dość wysoki, oraz co 2, 2,5, 5 lub 10 dB
    (albo wcale), gdy jest za niski. W mierniku cyfrowym (i własnym)
    wysoki miernik opisuje więcej wartości: co 1 dB od −20 do 0 oraz co
    5 dB między znacznikami co 10 dB poniżej −20 (−45, −55), zawsze
    rozmieszczonych równo i tylko tam, gdzie mieszczą się bez zlewania
    etykiet; pozostałe typy mierników zachowują etykiety wynikające z ich
    normy. Poziom odniesienia (−18 dBFS w mierniku cyfrowym) to grubsza
    biała kreska na obu podziałkach. Nic nie jest rysowane na słupkach ani
    między nimi, więc to, co widać w słupkach, to wyłącznie poziom,
    podtrzymanie szczytu i kolory.
  - Słupek jest zielony, żółty od poziomu ostrzeżenia (−9 dBFS) i czerwony
    od poziomu zagrożenia (−3 dBFS). Pozostałe typy mierników czerwienieją
    tam, gdzie ich skala (od 0 VU, od dozwolonego maksimum w PPM).
  - Mierniki K-System pokazują dwie części: pełny słupek to poziom średni
    (RMS), a ciemniejsza część nad nim sięga szczytu. Ich kolory to kolory
    K-System: zielony poniżej 0, bursztynowy od 0 do +4, czerwony powyżej.
  - Mierniki szczytu cyfrowego, K-System i własne przez chwilę utrzymują
    najwyższy poziom jako linię (podtrzymanie szczytu; jego długość to
    **Podtrzymanie szczytu** w [Ustawienia → Mierniki](settings.md#meters),
    a 0 je wyłącza). Mierniki PPM EBU, PPM DIN i VU nie mają podtrzymania.
  - Liczba nad miernikiem to najwyższy poziom od rozpoczęcia utworu, w
    dBFS, czerwona w strefie zagrożenia. Zostaje po zatrzymaniu i zaczyna
    od nowa, gdy zaczyna grać pozycja (następna albo ta sama jeszcze raz)
    lub gdy ją klikniesz.
  - Liczba pod miernikiem to głośność w LUFS (EBU R128), zielona w granicach
    ±1 LU od wartości docelowej (−23 LUFS).
  - Typ miernika i każdy poziom możesz zmienić w
    [Ustawienia → Mierniki](settings.md#meters).
  - **Odczyty powyżej 0 dBFS.** Miernik pokazuje to, co odtwarzacz wysyła, a
    to może przekraczać pełną skalę. Słupek zatrzymuje się na górze skali,
    więc ten sam czerwony oznacza 0 dBFS i wszystko powyżej; tylko liczba
    nad miernikiem mówi, jak daleko, wraz ze znakiem (na przykład `+3.5`).
    - Plik sam może zawierać poziomy powyżej pełnej skali (plik zmiennoprzecinkowy
      albo plik stratny, którego zdekodowane szczyty ją przekraczają).
    - Zmiana częstotliwości próbkowania może wytworzyć szczyty między
      próbkami: sygnał dotykający 0 dBFS pokazuje około +3 dBFS po
      konwersji 44,1 → 48 kHz. Opcja true peak odczytuje także te szczyty.
    - Miernik odczytuje każdy odtwarzacz osobno, a nie sumę na urządzeniu:
      dwa odtwarzacze na jednym wyjściu mogą sumować się powyżej pełnej
      skali, choć żaden miernik tego nie pokaże.
    - Nic w odtwarzaczu nie dodaje wzmocnienia powyżej 100 %. Urządzenie
      całkowitoliczbowe obcina sygnał na pełnej skali; urządzenie
      zmiennoprzecinkowe otrzymuje poziom bez zmian, a obcina go system
      dźwiękowy lub sterownik.
- **Suwak głośności** (na prawo od miernika, równy mu wysokością):
  przeciągaj go lub użyj kółka myszy, jeden krok na ząbek. Podpowiedź
  pokazuje poziom w dB; góra to 0 dB, a dół to cisza.
- **Tytuł, wykonawca** i wiersz **następny** z zielonym kwadratem. Gdy CUE
  jest włączone, pozycja odsłuchu wstępnego jest pokazana na niebiesko.

## Transport {#transport}

| Przycisk | Działanie |
|---|---|
| **Play / Następny** (duży) | Zatrzymany: uruchamia następny utwór. Na antenie: przechodzi płynnie do następnego utworu (czas wyciszania ustawia się w [Ustawieniach](settings.md)). Wstrzymany: wznawia. Gdy utwór na antenie jest ustawiony jako następny, Play uruchamia go od nowa ze zwykłym wyciszeniem. |
| **Stop** | Zatrzymuje od razu (z krótką rampą eliminującą trzask) |
| **Stop z wyciszeniem** | Wycisza i zatrzymuje |
| **Pauza** | Wstrzymuje lub wznawia; miga na bursztynowo, gdy jest wstrzymane |
| **Stop po** (trójkąt odtwarzania, potem kwadrat) | Zatrzymuje po zakończeniu bieżącego utworu, jednorazowo. W trybie SINGLE jest dostępny tylko wtedy, gdy bieżący utwór się powtarza: kończy powtarzanie, gdy skończy się grany właśnie przebieg. Aby zatrzymywać się po utworze za każdym razem, gdy jest grany, lub aby powtarzać utwór, użyj jego menu na playliście (zob. [Playlisty](playlists.md)) |
| **Poprzedni** (kreska i dwa trójkąty) | Na antenie: płynnie wraca do utworu, który ten odtwarzacz grał wcześniej, tak jak robi to Następny. Naciśnij ponownie, aby cofać się dalej. Utwór, który opuszczasz, staje się następnym. |
| **Od początku** (kreska i jeden trójkąt) | Wraca na początek bieżącego utworu (do jego cue-in). Wstrzymany odtwarzacz pozostaje wstrzymany. |

Przyciski, które w danej chwili nie mogą działać, są przyciemnione: Stop i
Od początku, gdy nic nie jest wczytane, Pauza i Stop z wyciszeniem w stanie
zatrzymania, Poprzedni, gdy nie ma wcześniejszego utworu albo podczas
wyciszania. Odtwarzacz pamięta ostatnie 50 odtworzonych utworów
(`players.history_len` w `config.json`, od 0 do 1000).

## Tryby {#modes}

- **CONT (ciągły):** w punkcie MIX odtwarzacz uruchamia następny utwór i
  nakłada go na koniec bieżącego. Zob. [Markery i
  miksowanie](markers-and-mixing.md).
- **SINGLE:** każdy utwór zatrzymuje się na końcu. *Stop po* nie jest
  dostępne w tym trybie, ponieważ każdy utwór i tak się zatrzymuje, z
  wyjątkiem sytuacji, gdy bieżący utwór się powtarza: wtedy kończy
  powtarzanie, gdy skończy się grany właśnie przebieg.

## Odliczanie {#countdown}

Duża liczba to czas pozostały do końca utworu (jego cue-out), z dziesiątymi
częściami sekundy. Czas, który upłynął, i całkowity są w wierszu pod
przebiegiem, po prawej. W ostatnich sekundach przed końcem (domyślnie 10,
ustawiane w Ustawieniach) odliczanie miga na czerwono.

## Przebieg {#waveform}

- Część już odtworzona jest narysowana kolorem przebiegu; reszta jest
  przyciemniona.
- Obrys pokazuje szczyty, słabo; pełne wypełnienie w jego wnętrzu to
  poziom średni (RMS). W głośnym utworze szczyty wypełniają całą wysokość,
  a wypełnienie nadal pokazuje, gdzie utwór jest cichszy lub głośniejszy.
- Niebieskie zacieniowane pole na początku oznacza **intro**, a plakietka
  odlicza jego czas. Intro jest widoczne tylko wtedy, gdy zostało ustawione.
- Pomarańczowe zacieniowane pole na końcu oznacza **outro**, z własnym
  odliczaniem.
- Przerywana bursztynowa linia z etykietą **MIX** oznacza miejsce, w którym
  w trybie ciągłym zaczyna się następny utwór. W trybie single jest
  przyciemniona.
- Rysowany jest cały plik. Ciche początek i koniec, które odtwarzanie
  pomija (przed cue-in i po cue-out), są narysowane ciemniej, z cienką
  linią w miejscu, gdzie odtwarzanie się zaczyna i kończy. Przy wyłączonej
  opcji **Używaj cue-in i cue-out** (Ustawienia → Odtwarzacze) nic nie jest
  ciemniejsze, a obie linie są przyciemnione: odtwarzanie trwa od początku
  do końca pliku, a cue-in i cue-out w tym podręczniku oznaczają te dwa
  końce.
- Najedź kursorem, aby zobaczyć czas pod wskaźnikiem. **Kliknij, aby
  przeskoczyć** w to miejsce. W zatrzymanym odtwarzaczu kliknięcie wybiera,
  skąd **Play** uruchomi następny utwór: głowica i odliczanie przesuwają
  się tam, a nic nie gra, dopóki nie naciśniesz Play. Wybór innego
  następnego utworu, jego przeniesienie lub usunięcie albo Stop wracają do
  cue-in; tak samo każdy inny sposób uruchomienia utworu, a Od początku,
  Poprzedni i automatyczne przejście zawsze używają cue-in. Kliknięcie
  przed cue-in (w ciemniejszym początku) wybiera cue-in. Kliknięcie w
  cue-out lub za nim (w ciemniejszym końcu) anuluje wcześniejszy wybór:
  Play zaczyna od cue-in. Kliknięcie to naciśnięcie i puszczenie bez
  przesunięcia wskaźnika o więcej niż kilka pikseli.
- **Naciśnięcie i przeciągnięcie** przesuwa powiększony widok wzdłuż
  utworu, jakbyś go chwycił. Przeciągnięcie nigdy nie przeskakuje, a bez
  powiększenia nic nie robi. Przeciągnięcie z Alt nadal edytuje markery.
- **Kółko myszy** nad przebiegiem: powiększa i pomniejsza wokół wskaźnika,
  aż do najdrobniejszego szczegółu, jaki ma analiza. **Shift+kółko** (albo
  kółko poziome) przesuwa wzdłuż utworu. Po powiększeniu widok podąża za
  pozycją odtwarzania, z wyjątkiem 10 sekund po powiększeniu lub
  przesunięciu (`ui.follow_current_grace_secs`; 0 wyłącza podążanie).
  **Pełny widok** w prawym górnym rogu, pełne oddalenie lub nowy utwór
  ponownie pokazują cały utwór.

## CUE (odsłuch wstępny) {#cue-pre-listen}

Naciśnięcie **CUE** (albo **Odsłuchaj na CUE** w menu utworu) odtwarza utwór
na wyjściu CUE odtwarzacza, na przykład w słuchawkach, bez dotykania wyjścia
na antenie, i otwiera małe **okno CUE** tego odtwarzacza. Można mieć
otwartych kilka okien, po jednym na odtwarzacz. Zob. [Ustawienia](settings.md),
aby wybrać urządzenie CUE. Odtwarzacz potrzebuje wyjścia Cue, które nie jest
jego wyjściem Main: bez niego **CUE** i **Odsłuchaj na CUE** są
przyciemnione, a po najechaniu kursorem pojawia się wyjaśnienie.

![Okno CUE odtwarzacza 4, odsłuchującego wstępnie swój następny utwór](../../images/guide/cue-window.png)

Okno pokazuje:

- tytuł i wykonawcę;
- przebieg całego pliku z pozycją CUE. Działa jak przebieg odtwarzacza:
  kliknięcie przeskakuje w dane miejsce, kółko powiększa, przeciągnięcie
  przesuwa, **Pełny widok**, odliczania intro i outro oraz markery intro,
  outro i MIX, które edytujesz tutaj tak jak w odtwarzaczu (zob. [Markery i
  miksowanie](markers-and-mixing.md)). CUE odtwarza cały plik, więc nic nie
  jest narysowane ciemniej, cue-in i cue-out to przyciemnione linie, a outro
  odlicza do końca pliku. Powiększenie jest osobne: przebieg odtwarzacza się
  nie rusza. Gdy CUE gra, powiększony widok podąża za jego pozycją tak jak w
  odtwarzaczu; wstrzymane CUE zachowuje ustawiony widok, więc możesz
  powiększać i umieszczać markery. CUE uruchomione po zamknięciu okna albo
  na innym utworze pokazuje cały plik;
- czas, który upłynął, i czas pozostały do końca pliku (CUE odtwarza całe
  pliki);
- **Wstrzymaj CUE** / **Wznów CUE**, **Zatrzymaj CUE** i **Ustaw jako
  następny**. **Ustaw jako następny** ustawia odsłuchiwany utwór jako następny w odtwarzaczu i
  pozwala CUE grać dalej. Jest przyciemnione, gdy utwór już jest następnym.
  Jeśli odsłuchiwany utwór jest tym na antenie, zagra jeszcze raz po
  zakończeniu bieżącego przebiegu.

Gdy CUE jest wstrzymane, jego przycisk wstrzymania (pokazany jako **Wznów
CUE**) miga na bursztynowo, tak jak własny przycisk odtwarzacza.

Przeskok we wstrzymanym CUE zostawia je wstrzymane. Przycisk zamknięcia okna
albo **Zatrzymaj CUE** zatrzymuje CUE.

Gdy CUE działa, ustawienie następnego utworu (dwuklik) lub pojedyncze
kliknięcie wiersza przenosi CUE na ten utwór, od jego cue-in; jeśli było
wstrzymane, gra ponownie. Utwór, którego plik nie istnieje lub jest
nieczytelny, zostawia CUE tam, gdzie jest.
