# Cartutxera

La cartutxera és el panell de botons sota els reproductors, o al seu costat
(vegeu [Disposició](layout.md#move-and-resize-the-cartwall)). Cada botó, un
**cartutx**, reprodueix un so a l'instant: jingles, efectes, falques. Els
cartutxos sonen per les seves pròpies sortides, independentment dels
reproductors.

![La cartutxera amb un cartutx sonant](../../images/guide/cartwall.png)

## Ús {#using-it}

- **Fes clic en un cartutx** per disparar-lo. **Fes-hi clic de nou** per
  aturar-lo.
- **Arrossega pistes des de la [finestra de la biblioteca](library.md)** a un
  cartutx per posar-hi la primera (substituint el seu fitxer quan és aturat).
  Un cartutx que sona o és en CUE refusa el que s'hi deixa anar.
- **Aturar-ho tot** (a l'extrem dret de la barra) atura tots els cartutxos que
  sonen, a totes les pàgines. L'etiqueta mostra quants en sonen, com a
  **Aturar-ho tot (2)**; si no en sona cap, apareix atenuat i sense comptador.
- Mentre un cartutx sona, la seva vora i una barra que creix des de l'esquerra
  a mesura que sona prenen el [color](#setting-up-carts) del cartutx (vermell
  per a un cartutx sense color), una línia fina marca la vora reproduïda sobre
  la forma d'ona, i el temps compta enrere. La línia es mou suaument, fins i
  tot en un cartutx curt.
- Quan el fitxer d'un cartutx s'ha analitzat, la seva forma d'ona es mostra
  tènuement darrere del nom, de manera que pots distingir un so que colpeja de
  seguida d'un que entra en fos. Cobreix el que reprodueix el cartutx, des del
  seu cue-in fins al seu cue-out.
- Els cartutxos se **superposen** per defecte: disparar-ne un segon no atura
  el primer. Un cartutx configurat amb **Aturar els altres cartutxos en
  disparar-lo** atura primer tots els altres cartutxos en antena, a qualsevol
  pàgina.
- Un cartutx configurat amb **En bucle** torna a començar des del seu cue-in
  quan arriba al final, sense pausa, fins que l'aturis.
- **Clic dret** en un cartutx per veure més opcions:

| Element | Acció |
|---|---|
| Preescoltar al CUE | El reprodueix a la sortida CUE de la cartutxera (atenuat quan la cartutxera no té sortida Cue). A la sortida d'antena pregunta primer mentre hi sona alguna cosa (vegeu [CUE](players.md#cue-pre-listen)) |
| Aturar | L'atura |
| Editar… | L'obre a Configuració |

El menú d'un cartutx buit només té **Editar…**, per triar-ne el fitxer.

- **Pàgines:** les pestanyes al costat de **CARTUTXERA** canvien de pàgina. Un
  punt vermell indica que un cartutx d'aquella pàgina està sonant.
- Fes clic a **CARTUTXERA** per plegar la cartutxera o desplegar-la de nou.
  Plegada al costat dels reproductors és una franja estreta amb una fletxa per
  desplegar-la i, mentre sona un cartutx, un punt vermell.
- Sota els reproductors, tan alta com necessiten les seves files, els botons
  s'encongeixen quan la finestra és baixa (fins a una alçada mínima) perquè
  càpiguen totes les files configurades. Quan canvies la mida de la
  cartutxera, o al costat dels reproductors, els botons creixen per omplir-la,
  i els noms i els temps conserven la seva mida al mig de cada botó. En tots
  dos casos la cartutxera només es desplaça quan ni els botons més petits hi
  caben.

| Aspecte del botó | Significat |
|---|---|
| Punt violeta | Jingle |
| Punt ambre | Efecte |
| Punt gris | Falca (publicitat) |
| Vora, tinta i punt de color | El color del cartutx, triat a Configuració: el punt pren el color en lloc del del tipus |
| ↻ després del tipus | En bucle |
| ✋ després del tipus | Atura els altres cartutxos en disparar-se |
| Fitxer amb una creu / senyal d'advertència | El fitxer falta / no es pot decodificar; posa-hi el ratolí a sobre del cartutx per veure'n el motiu i el camí. Un fitxer que falta es torna a buscar cada 30 s (`tuning.missing_recheck_ms`). |
| «Buit», atenuat | Sense fitxer assignat |

Els cartutxos usen els mateixos marcadors que les pistes. Comencen al seu
cue-in i acaben al seu cue-out, que pots editar a la forma d'ona d'un
reproductor quan el fitxer hi és carregat.

## Teclat {#keyboard}

Per defecte, **F1**…**F12** disparen els cartutxos 1–12 de la pàgina
mostrada, i **Ctrl+Space** atura tots els cartutxos (el mateix que **Aturar-ho
tot**; també atura un cartutx que estàs preescoltant al CUE, fins i tot quan
no en sona cap). Vegeu [Teclat](keyboard.md) per canviar-les.

La drecera que plega o desplega la cartutxera no fa res mentre la cartutxera és
amagada des de la barra superior (vegeu [Disposició](layout.md#show-and-hide-panels)).

## Configurar els cartutxos {#setting-up-carts}

Ves a **Configuració → Cartutxera**:

- **Pàgines:** crear, canviar el nom, eliminar (l'última pàgina no es pot
  eliminar), i definir la mida de la graella (files × columnes). Es refusa
  una graella més petita si perdria cartutxos que tenen fitxer. Una mida
  escrita al quadre al costat d'un control lliscant s'aplica quan prems Enter
  o surts del quadre; Escape la cancel·la.
- **Importar… / Exportar…** desen una pàgina en un fitxer `.cartpage.json` i
  la tornen a carregar, per exemple per compartir-la entre estudis. Els camins
  de fitxer relatius es resolen respecte a la carpeta del fitxer. Els colors viatgen amb la pàgina; un
  fitxer d'una versió més antiga s'importa sense ells.
- **Cartutxos:** fes clic en un cartutx de la graella i defineix-ne el nom, el
  fitxer (**Triar…** o **Treure**), el tipus, el **Color**, **En bucle** i
  **Aturar els altres cartutxos en disparar-lo**. **Color** ofereix sense color
  i vuit colors (vermell, taronja, ambre, verd, turquesa, blau, violeta, rosa):
  dona un color a totes les falques i un altre a les identificacions de
  l'emissora per trobar-les d'un cop d'ull. Un cartutx buit conserva el seu
  color, i treure el fitxer d'un cartutx no el treu. Un cartutx amb fitxer
  també té **Excloure de la normalització** i **Ajust**, els ajustos de
  [normalització de la sonoritat](loudness.md#per-track-exclude-and-trim) de la
  seva pista, amb la mesura sota l'etiqueta.

Les sortides Main i Cue pròpies de la cartutxera són a **Configuració →
Sortides d'àudio** (la fila **Cartutxera**).
