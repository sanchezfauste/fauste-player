# Cartwall

Cartwall to pasek przycisków pod odtwarzaczami. Każdy przycisk, czyli
**cart**, natychmiast odtwarza jeden dźwięk: jingle, efekty, spoty. Carty
grają na własnych wyjściach, niezależnie od odtwarzaczy.

![Cartwall z jednym grającym cartem](../../images/guide/cartwall.png)

## Obsługa {#using-it}

- **Kliknij cart**, aby go odpalić. **Kliknij go ponownie**, aby go
  zatrzymać.
- **Zatrzymaj wszystko** (prawy koniec paska) zatrzymuje każdy grający cart,
  na każdej stronie. Etykieta pokazuje, ile ich gra, np. **Zatrzymaj
  wszystko (2)**; gdy żaden nie gra, przycisk jest przyciemniony i nie
  pokazuje liczby.
- Gdy cart gra, jego obramowanie robi się czerwone, czerwony pasek kurczy
  się w miarę odtwarzania, a czas odlicza w dół.
- Carty domyślnie się **nakładają**: odpalenie drugiego nie zatrzymuje
  pierwszego. Cart z włączoną opcją **Zatrzymuj inne carty po odpaleniu**
  najpierw zatrzymuje wszystkie inne carty na antenie, na dowolnej stronie.
- Cart z ustawioną opcją **Pętla** po dotarciu do końca zaczyna od nowa od
  swojego cue-in, bez przerwy, dopóki go nie zatrzymasz.
- **Kliknięcie prawym przyciskiem** carta daje więcej opcji:

| Pozycja | Działanie |
|---|---|
| Odsłuchaj na CUE | Odtwarza go na wyjściu CUE cartwalla (przyciemnione, gdy cartwall nie ma wyjścia Cue innego niż Main) |
| Zatrzymaj | Zatrzymuje go |
| Edytuj… | Otwiera go w Ustawieniach |

Menu pustego carta zawiera tylko **Edytuj…**, aby wybrać jego plik.

- **Strony:** karty obok **CARTWALL** przełączają strony. Czerwona kropka
  pokazuje, że gra cart na tej stronie.
- Kliknij **CARTWALL**, aby zwinąć pasek lub rozwinąć go ponownie.
- Gdy okno jest niskie, przyciski się zmniejszają (do minimalnej wysokości),
  tak aby zmieściły się wszystkie skonfigurowane wiersze; cartwall przewija
  się tylko wtedy, gdy nie mieszczą się nawet najmniejsze przyciski.

| Wygląd przycisku | Znaczenie |
|---|---|
| Fioletowa kropka | Jingiel |
| Bursztynowa kropka | Efekt |
| Szara kropka | Spot (reklamowy) |
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

## Konfigurowanie cartów {#setting-up-carts}

Przejdź do **Ustawienia → Cartwall**:

- **Strony:** twórz, zmieniaj nazwy, usuwaj (ostatniej strony nie można
  usunąć) i ustawiaj rozmiar siatki (wiersze × kolumny). Mniejsza siatka jest
  odrzucana, jeśli usunęłaby carty z przypisanym plikiem.
- **Importuj… / Eksportuj…** zapisują stronę do pliku `.cartpage.json` i
  wczytują ją z powrotem, na przykład aby dzielić się nią między studiami.
  Ścieżki względne plików są rozwiązywane względem folderu pliku.
- **Carty:** kliknij cart w siatce, a następnie ustaw jego nazwę, plik
  (**Wybierz…** lub **Wyczyść**), typ, **Pętla** oraz **Zatrzymuj inne carty
  po odpaleniu**.

Własne wyjścia Main i Cue cartwalla są w **Ustawienia → Wyjścia audio**
(wiersz **Cartwall**).
