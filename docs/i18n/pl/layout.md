# Układ

Ułóż główne okno pod swoją zmianę: nadaj odtwarzaczom i cartwallowi żądane
rozmiary, umieść cartwall pod odtwarzaczami albo obok nich, ukryj panele,
których nie używasz, i na chwilę zmaksymalizuj jeden z nich. Układ jest
zapisywany z sesją i wraca przy następnym uruchomieniu; nowa instalacja
startuje z każdym odtwarzaczem tej samej szerokości i cartwallem pod nimi.

## Pokazywanie i ukrywanie paneli {#show-and-hide-panels}

Kafelki po lewej stronie górnego paska, **Ramówka** i **Muzyka**
([ramówka](automation.md#the-schedule-column) i
[wypełniacz muzyczny](automation.md#the-music-filler), oba ukryte, dopóki ich
nie pokażesz), **P1** … **Pn** i **Cartwall**, pokazują, które panele są na
ekranie: podświetlony kafelek oznacza panel pokazany. Kliknij kafelek, aby ukryć lub pokazać jego panel. Przy wielu
odtwarzaczach kafelki przewijają się w bok (przewiń kółkiem myszy nad nimi), a
przycisk układu pozostaje w zasięgu wzroku. Pozostałe panele zajmują
zwolnione miejsce, a ponownie pokazany odtwarzacz wraca ze swoją szerokością.

Ukrycie tylko zdejmuje panel z ekranu. Ukryty odtwarzacz nadal gra, a
sterują nim nadal jego klawisz numeryczny, MIDI, zdalne API, jego okno CUE i
automatyka. Kafelek pokazuje stan swojego panelu, niezależnie od tego, czy
panel jest pokazany, czy ukryty: czerwona kropka, gdy odtwarzacz gra (albo
cartwall, gdy gra cart), i dwie bursztynowe kreski, jak na przycisku Pauza,
gdy odtwarzacz jest wstrzymany.

Ukrycie odtwarzacza czyści jego zaznaczony utwór, więc **Delete** nie może
usunąć utworu, którego nie widzisz. Pliki upuszczone na okno poza listą
trafiają do ostatnio używanego odtwarzacza albo do pierwszego pokazywanego;
gdy każdy odtwarzacz jest ukryty, są odrzucane, a pasek stanu o tym informuje.

Kafelek **Biblioteka**, za nimi, otwiera i zamyka [okno biblioteki](library.md),
które unosi się nad panelami; nie jest panelem, więc zasada ostatniego
podświetlonego kafelka poniżej go nie dotyczy. Nie jest pokazywany na
telefonie.

Co najmniej jeden panel pozostaje pokazany: ostatniego podświetlonego
kafelka nie można kliknąć. Gdy cartwall jest ukryty, skrót zwijający lub
rozwijający go nic nie robi; kliknij **Cartwall**, aby go pokazać.

## Zmiana rozmiaru odtwarzaczy {#resize-the-players}

Przeciągnij przerwę między dwoma odtwarzaczami, aby przenieść szerokość z
jednego na drugi. Pozostałe odtwarzacze się nie przesuwają, a żaden
odtwarzacz nie staje się węższy, niż potrzebują jego kontrolki. Kliknij
przerwę dwukrotnie, aby ponownie nadać każdemu odtwarzaczowi tę samą
szerokość.

Gdy okno jest za wąskie na każdy pokazywany odtwarzacz w jego najmniejszym
rozmiarze, rząd odtwarzaczy przewija się w bok, jak zawsze.

## Przenoszenie i zmiana rozmiaru cartwalla {#move-and-resize-the-cartwall}

Cartwall leży pod odtwarzaczami albo po lewej lub prawej stronie okna, na
całej jego wysokości: wybierz **Cartwall na dole**, **Cartwall po lewej** lub
**Cartwall po prawej** w [menu układu](#the-layout-menu). Pod odtwarzaczami
jest tak wysoki, jak potrzebują jego wiersze, najwyżej sześć dziesiątych
wysokości między górnym paskiem a paskiem stanu; obok nich zajmuje na początku
trzy dziesiąte szerokości.

Cartwall nigdy nie jest węższy niż jego nagłówek (**Zatrzymaj wszystko**,
tytuł i karta strony) ani niż jego kolumny przycisków w najmniejszym
rozmiarze. Gdy okno jest za wąskie na niego i każdy pokazywany odtwarzacz w
najmniejszym rozmiarze, rząd odtwarzaczy przewija się w bok.

Przeciągnij przerwę między cartwallem a odtwarzaczami, aby go powiększyć lub
zmniejszyć: w górę lub w dół, gdy jest pod odtwarzaczami, w bok, gdy jest
obok nich. Odtwarzacze zachowują miejsce, którego potrzebują ich kontrolki.
Kliknij przerwę dwukrotnie, aby wrócić do rozmiaru początkowego: tak wysoki,
jak potrzebują jego wiersze, pod odtwarzaczami, trzy dziesiąte szerokości obok
nich. Rozmiar jest zapisywany z układem; przeniesienie cartwalla zaczyna go
znów od rozmiaru początkowego.

Gdy zmienisz rozmiar cartwalla albo gdy leży obok odtwarzaczy, jego przyciski
rosną, by go wypełnić; nazwy i czasy zachowują swój rozmiar, pośrodku każdego
przycisku. Nazwa zawija się między słowami; słowo zbyt długie na swój
przycisk kończy się wielokropkiem (…) zamiast łamać się między wierszami. Gdy
cartwall jest mniejszy, niż potrzebują jego wiersze i kolumny przy
najmniejszych przyciskach, przewija się.

Kliknij **CARTWALL**, aby zwinąć cartwall. Pod odtwarzaczami zostaje tylko jego
nagłówek; obok nich staje się wąskim paskiem o wysokości okna, ze strzałką do
ponownego rozwinięcia i, gdy cart gra, czerwoną kropką. Rozwinięty odzyskuje
swój rozmiar. Gdy jest zwinięty, przerwa obok niego nic nie robi.

## Maksymalizowanie panelu {#maximize-a-panel}

Nagłówek każdego odtwarzacza oraz cartwalla kończy się dwoma małymi
przyciskami: **Ukryj** (minus) i **Maksymalizuj** (rogi skierowane na
zewnątrz; **Przywróć**, rogi skierowane do wewnątrz, na zmaksymalizowanym
panelu). Najedź kursorem, aby zobaczyć nazwę. Są przyciemnione, gdy układ jest
zablokowany, **Ukryj** na ostatnim pokazywanym panelu, a **Maksymalizuj** przy
jednym pokazywanym panelu i na zwiniętym cartwallu. Te same dwie akcje są w
menu prawego kliknięcia na tle nagłówka (nie na jednym z jego przycisków):

- **Maksymalizuj** daje temu panelowi całe miejsce: pozostałe panele
  zmniejszają się do swojego minimum. To tylko na chwilę i nie jest zapisywane.
  **Przywróć** w tym samym menu to kończy, podobnie jak ukrycie lub pokazanie
  panelu, zablokowanie układu, przeniesienie cartwalla, zmiana liczby
  odtwarzaczy oraz przeciągnięcie lub dwukrotne kliknięcie przerwy między
  panelami; dawne rozmiary wracają. Zwiniętego cartwalla nie można
  zmaksymalizować, a zwinięcie zmaksymalizowanego cartwalla to kończy.
- **Ukryj** ukrywa panel, tak jak jego kafelek na górnym pasku.

## Menu układu {#the-layout-menu}

Przycisk układu, na prawo od kafelków, otwiera menu z pozycjami:

- **Cartwall na dole**, **Cartwall po lewej**, **Cartwall po prawej**: gdzie
  leży cartwall;
- **Zablokuj układ**: gdy jest zaznaczone, nic nie zmienia układu przez
  przypadek: kafelki, pozostałe pozycje tego menu, przerwy między panelami
  oraz **Maksymalizuj** i **Ukryj** w nagłówkach nic nie robią. Odtwarzanie
  oraz zwijanie i rozwijanie cartwalla działają jak zwykle. Blokada jest
  zapisywana z układem;
- **Równa szerokość odtwarzaczy**: każdy odtwarzacz tej samej szerokości;
- **Resetuj układ**: powrót do układu nowej instalacji.
