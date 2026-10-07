# Markery i miksowanie

Każdy utwór ma do pięciu **markerów**, w sekundach:

| Marker | Znaczenie | Jak jest ustawiany |
|---|---|---|
| Cue in | Gdzie zaczyna się odtwarzanie | Automatycznie: tuż przed pierwszym dźwiękiem powyżej progu przycinania |
| Cue out | Gdzie kończy się utwór | Automatycznie: tuż po ostatnim dźwięku powyżej progu przycinania |
| MIX (początek segue) | Gdzie w trybie ciągłym zaczyna się następny utwór | Automatycznie (zob. niżej) |
| Początek outro | Gdzie zaczyna się zakończenie utworu | Automatycznie (zob. niżej) |
| Koniec intro | Koniec wstępu, na który się mówi | Ręcznie albo z tagu `INTRO` w pliku |

Markery ustawione ręcznie zawsze wygrywają: nowa analiza nigdy ich nie
zastępuje.

## Edycja markerów {#editing-markers}

Na przebiegu odtwarzacza lub na przebiegu jego okna CUE (to samo menu i te
same uchwyty; zmiana widoczna jest w obu naraz):

- **Kliknij prawym przyciskiem** w miejscu, gdzie chcesz marker, i wybierz
  **Ustaw tutaj cue-in**, **Ustaw tutaj koniec intro**, **Ustaw tutaj
  początek outro**, **Ustaw tutaj punkt MIX** lub **Ustaw tutaj cue-out**.
  **Przywróć automatyczne znaczniki** usuwa markery, które umieściłeś, a
  utwór jest analizowany ponownie.
- **Przytrzymaj Alt** (Option w macOS): na markerach pojawiają się uchwyty.
  Przeciągnij jeden, aby go przesunąć; podczas przeciągania widać czas.
  Przeciągnięcie nigdy nie przesuwa głowicy odtwarzania.

Cue-in musi pozostać przed cue-out. Pozostałe markery są utrzymywane między
nimi. Zmiany w utworze na antenie obowiązują od razu przy jego następnym
przejściu.

## Tag INTRO {#the-intro-tag}

Plik może nieść czas intro w tagu `INTRO`, jako sekundy (`12.5`) albo
`m:ss`. Nazwę można pisać dowolną wielkością liter (`INTRO`, `Intro`). Może
to być ramka tekstowa użytkownika ID3v2 (MP3, WAV, AIFF, DSF), komentarz
Vorbis, Opus lub FLAC, element APE (WavPack, Monkey's Audio) albo atom
freeform MP4. Jest odczytywany podczas analizy. Ręcznie ustawiony koniec
intro nadal wygrywa.

## Jak znajdowane są markery automatyczne {#how-the-automatic-markers-are-found}

Analiza mierzy szczyty utworu w krokach co 10 ms oraz jego głośność w
krótkich oknach (domyślnie 50 ms).

- **Cue in / cue out:** pomijana jest tylko niemal-cisza na początku i na
  końcu: wszystko, czego szczyt osiąga *próg przycinania* (domyślnie
  −60 dBFS), na dowolnym kanale, jest zachowywane, z *marginesem
  przycinania* (domyślnie 20 ms) wokół. Łagodne narastanie, ciche zakończenia
  i krótkie dźwięki nigdy nie są obcinane.
- **MIX:** analiza znajduje ostatni punkt, w którym utwór jest jeszcze mniej
  niż o *spadek poziomu dla segue* (domyślnie 15 dB) poniżej własnej typowej
  głośności, dzięki czemu głośne i ciche mastery z takim samym wyciszeniem
  miksują się tak samo. Punkt ten nigdy nie leży dalej niż *maksymalna
  długość miksu* (domyślnie 4 s) przed cue-out, więc nakładanie pozostaje
  krótkie.
- **Outro:** analiza skanuje wstecz od cue-out i znajduje miejsce, gdzie
  poziom spada o więcej niż *spadek poziomu outro* (domyślnie 6 dB) poniżej
  mediany głośności utworu. Outro nigdy nie jest dłuższe niż domyślnie 30 s.
- Utwory krótsze niż *minimalna długość dla znaczników miksu i outro*
  (domyślnie 60 s), takie jak jingle i reklamy, nie dostają MIX ani outro.

### Długie nagrania {#long-recordings}

Cały program (jedna, cztery lub więcej godzin) jest analizowany jak
piosenka, w razie potrzeby podczas odtwarzania: 4-godzinny plik FLAC lub
Opus zajmuje poniżej minuty na obecnym komputerze, a pamięć nie rośnie wraz
z długością. Przeskok w dowolne miejsce, nawet blisko końca, jest
natychmiastowy. Przebieg i markery są trzymane w pamięci podręcznej analizy
do około 16 godzin dźwięku; dłuższy plik też działa, ale jest analizowany od
nowa przy każdym uruchomieniu Fauste Player.

Wszystkie te wartości są w **Ustawienia → Analiza**. Po ich zmianie utwory
są analizowane ponownie automatycznie.

## Co odtwarzacz z nimi robi {#what-the-player-does-with-them}

- **Tryb ciągły z włączonym automatycznym miksem:** w punkcie MIX następny
  utwór startuje z pełnym poziomem, podczas gdy bieżący wycisza się aż do
  swojego cue-out. Nakładanie jest dokładne co do próbki.
- **Tryb ciągły bez punktu MIX** lub z wyłączonym automatycznym miksem:
  następny utwór startuje dokładnie w cue-out, bez przerwy.
- **Tryb single** lub **Stop po**: odtwarzacz zatrzymuje się w cue-out.
- **Naciśnięcie Play na antenie:** następny utwór startuje od razu, a
  bieżący wycisza się przez *czas wyciszania* (domyślnie 1 s).
- **Wyłączone Używaj cue-in i cue-out** (Ustawienia → Odtwarzacze):
  każdy odtwarzacz gra każdy utwór od 0 do końca pliku. Cue-in i cue-out,
  automatyczne i ręczne, są zachowywane, a przebieg rysuje je jako
  przyciemnione linie. Punkt MIX, intro i outro nadal działają w obrębie
  całego pliku; **Automatyczny miks w punkcie MIX** to osobny przełącznik.
  Odliczania, kolumna czasu, sumy playlist w Ustawieniach i czasy w zdalnym
  API podążają za tym samym zakresem. Carty zawsze używają własnych cue-in
  i cue-out. Zmiana ustawienia nigdy nie restartuje, nie przewija ani nie
  zatrzymuje grającego utworu; następny utwór jest przygotowywany od nowa.

Utwór można odtworzyć, zanim jego analiza się skończy. Do tego czasu gra od
początku do końca pliku, bez punktu MIX.
