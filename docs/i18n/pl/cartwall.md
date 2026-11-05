# Cartwall

Cartwall to panel przycisków pod odtwarzaczami lub obok nich (zobacz
[Układ](layout.md#move-and-resize-the-cartwall)). Każdy przycisk, czyli
**cart**, natychmiast odtwarza jeden dźwięk: jingle, efekty, spoty. Carty
grają na własnych wyjściach, niezależnie od odtwarzaczy.

![Cartwall z jednym grającym cartem](../../images/guide/cartwall.png)

## Obsługa {#using-it}

- **Kliknij cart**, aby go odpalić. **Kliknij go ponownie**, aby go
  zatrzymać.
- **Przeciągnij utwory z [okna biblioteki](library.md)** na cart, aby wstawić
  do niego pierwszy z nich (zastępuje plik carta, gdy ten jest zatrzymany).
  Cart, który gra lub jest na CUE, odmawia upuszczenia.
- **Zatrzymaj wszystko** (prawy koniec paska) zatrzymuje każdy grający cart,
  na każdej stronie. Etykieta pokazuje, ile ich gra, np. **Zatrzymaj
  wszystko (2)**; gdy żaden nie gra, przycisk jest przyciemniony i nie
  pokazuje liczby.
- Gdy cart gra, jego obramowanie i pasek rosnący od lewej w miarę
  odtwarzania przybierają [kolor](#setting-up-carts) carta (czerwony dla
  carta bez koloru), cienka linia oznacza odtworzoną krawędź na przebiegu, a
  jego czas odlicza w dół. Linia przesuwa się płynnie, nawet na krótkim
  carcie.
- Gdy plik carta zostanie przeanalizowany, jego przebieg pojawia się blado za
  nazwą, dzięki czemu odróżnisz dźwięk uderzający od razu od takiego, który
  narasta. Obejmuje to, co cart gra, od jego cue-in do cue-out.
- Carty domyślnie się **nakładają**: odpalenie drugiego nie zatrzymuje
  pierwszego. Cart z włączoną opcją **Zatrzymuj inne carty po odpaleniu**
  najpierw zatrzymuje wszystkie inne carty na antenie, na dowolnej stronie.
- Cart z ustawioną opcją **Pętla** po dotarciu do końca zaczyna od nowa od
  swojego cue-in, bez przerwy, dopóki go nie zatrzymasz.
- **Kliknięcie prawym przyciskiem** carta daje więcej opcji:

| Pozycja | Działanie |
|---|---|
| Odsłuchaj na CUE | Odtwarza go na wyjściu CUE cartwalla (przyciemnione, gdy cartwall nie ma wyjścia Cue). Na wyjściu antenowym najpierw pyta, gdy coś tam gra (zob. [CUE](players.md#cue-pre-listen)) |
| Zatrzymaj | Zatrzymuje go |
| Edytuj… | Otwiera go w Ustawieniach |

Menu pustego carta zawiera tylko **Edytuj…**, aby wybrać jego plik.

- **Strony:** karty obok **CARTWALL** przełączają strony. Czerwona kropka
  pokazuje, że gra cart na tej stronie.
- Kliknij **CARTWALL**, aby zwinąć cartwall lub rozwinąć go ponownie. Zwinięty
  obok odtwarzaczy jest wąskim paskiem ze strzałką do rozwinięcia i, gdy cart
  gra, czerwoną kropką.
- Pod odtwarzaczami, tak wysoki, jak potrzebują jego wiersze, przyciski się
  zmniejszają, gdy okno jest niskie (do minimalnej wysokości), tak aby
  zmieściły się wszystkie skonfigurowane wiersze. Gdy zmienisz rozmiar
  cartwalla albo gdy jest obok odtwarzaczy, przyciski rosną, by go wypełnić, a
  nazwy i czasy zachowują swój rozmiar pośrodku każdego przycisku. W obu
  przypadkach cartwall przewija się tylko wtedy, gdy nie mieszczą się nawet
  najmniejsze przyciski.

| Wygląd przycisku | Znaczenie |
|---|---|
| Fioletowa kropka | Jingiel |
| Bursztynowa kropka | Efekt |
| Szara kropka | Spot (reklamowy) |
| Kolorowe obramowanie, odcień i kropka | Kolor carta, wybrany w Ustawieniach: kropka przyjmuje ten kolor zamiast koloru typu |
| ↻ po typie | Zapętlony |
| ✋ po typie | Po odpaleniu zatrzymuje inne carty |
| Plik z krzyżykiem / znak ostrzeżenia | Pliku brakuje / nie można go zdekodować; najedź na cart, aby zobaczyć przyczynę i ścieżkę. Brakującego pliku szuka się ponownie co 30 s (`tuning.missing_recheck_ms`). |
| „Pusty”, przyciemniony | Nie przypisano pliku |

Carty używają tych samych markerów co utwory. Zaczynają od swojego cue-in i
kończą na swoim cue-out, które możesz edytować na przebiegu odtwarzacza, gdy
plik jest tam wczytany.

## Klawiatura {#keyboard}

Domyślnie **F1**…**F12** odpalają carty 1–12 pokazywanej strony, a
**Ctrl+Space** zatrzymuje każdy cart (tak jak **Zatrzymaj wszystko**;
zatrzymuje też cart odsłuchiwany na CUE, nawet gdy żaden cart nie gra).
Zob. [Klawiatura](keyboard.md), aby je zmienić.

Skrót zwijający lub rozwijający cartwall nic nie robi, gdy cartwall jest
ukryty na górnym pasku (zob. [Układ](layout.md#show-and-hide-panels)).

## Konfigurowanie cartów {#setting-up-carts}

Przejdź do **Ustawienia → Cartwall**:

- **Strony:** twórz, zmieniaj nazwy, usuwaj (ostatniej strony nie można
  usunąć) i ustawiaj rozmiar siatki (wiersze × kolumny). Mniejsza siatka jest
  odrzucana, jeśli usunęłaby carty z przypisanym plikiem. Rozmiar wpisany w
  pole obok suwaka obowiązuje po naciśnięciu Enter lub opuszczeniu pola;
  Escape go anuluje.
- **Importuj… / Eksportuj…** zapisują stronę do pliku `.cartpage.json` i
  wczytują ją z powrotem, na przykład aby dzielić się nią między studiami.
  Ścieżki względne plików są rozwiązywane względem folderu pliku. Kolory
  podróżują ze stroną; plik ze starszej wersji importuje się bez nich.
- **Carty:** kliknij cart w siatce, a następnie ustaw jego nazwę, plik
  (**Wybierz…** lub **Wyczyść**), typ, **Kolor**, **Pętla** oraz **Zatrzymuj
  inne carty po odpaleniu**. **Kolor** oferuje brak koloru i osiem kolorów
  (czerwony, pomarańczowy, bursztynowy, zielony, turkusowy, niebieski,
  fioletowy, różowy): nadaj jeden kolor wszystkim spotom, a inny sygnałom
  stacji, by znajdować je na pierwszy rzut oka. Pusty cart zachowuje swój
  kolor, a wyczyszczenie pliku carta go nie usuwa. Cart z plikiem ma też
  **Wyłącz z normalizacji** i **Korekta**, czyli ustawienia
  [normalizacji głośności](loudness.md#per-track-exclude-and-trim) jego
  utworu, z pomiarem pod etykietą.

Własne wyjścia Main i Cue cartwalla są w **Ustawienia → Wyjścia audio**
(wiersz **Cartwall**).
