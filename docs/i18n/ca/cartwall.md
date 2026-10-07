# Cartutxera

La cartutxera és la franja de botons sota els reproductors. Cada botó, un
**cartutx**, reprodueix un so a l'instant: jingles, efectes, falques. Els
cartutxos sonen per les seves pròpies sortides, independentment dels
reproductors.

![La cartutxera amb un cartutx sonant](../../images/guide/cartwall.png)

## Ús {#using-it}

- **Fes clic en un cartutx** per disparar-lo. **Fes-hi clic de nou** per
  aturar-lo.
- **Aturar-ho tot** (a l'extrem dret de la barra) atura tots els cartutxos que
  sonen, a totes les pàgines. L'etiqueta mostra quants en sonen, com a
  **Aturar-ho tot (2)**; si no en sona cap, apareix atenuat i sense comptador.
- Mentre un cartutx sona, la vora es posa vermella, una barra vermella
  s'encongeix a mesura que sona, i el temps compta enrere.
- Els cartutxos se **superposen** per defecte: disparar-ne un segon no atura
  el primer. Un cartutx configurat amb **Aturar els altres cartutxos en
  disparar-lo** atura primer tots els altres cartutxos en antena, a qualsevol
  pàgina.
- Un cartutx configurat amb **En bucle** torna a començar des del seu cue-in
  quan arriba al final, sense pausa, fins que l'aturis.
- **Clic dret** en un cartutx per veure més opcions:

| Element | Acció |
|---|---|
| Preescoltar al CUE | El reprodueix a la sortida CUE de la cartutxera (atenuat quan la cartutxera no té una sortida Cue apart de la seva sortida Main) |
| Aturar | L'atura |
| Editar… | L'obre a Configuració |

El menú d'un cartutx buit només té **Editar…**, per triar-ne el fitxer.

- **Pàgines:** les pestanyes al costat de **CARTUTXERA** canvien de pàgina. Un
  punt vermell indica que un cartutx d'aquella pàgina està sonant.
- Fes clic a **CARTUTXERA** per plegar la franja o desplegar-la de nou.
- Quan la finestra és baixa, els botons s'encongeixen (fins a una alçada
  mínima) perquè càpiguen totes les files configurades; la cartutxera només
  es desplaça quan ni els botons més petits hi caben.

| Aspecte del botó | Significat |
|---|---|
| Punt violeta | Jingle |
| Punt ambre | Efecte |
| Punt gris | Falca (publicitat) |
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

## Configurar els cartutxos {#setting-up-carts}

Ves a **Configuració → Cartutxera**:

- **Pàgines:** crear, canviar el nom, eliminar (l'última pàgina no es pot
  eliminar), i definir la mida de la graella (files × columnes). Es refusa
  una graella més petita si perdria cartutxos que tenen fitxer.
- **Importar… / Exportar…** desen una pàgina en un fitxer `.cartpage.json` i
  la tornen a carregar, per exemple per compartir-la entre estudis. Els camins
  de fitxer relatius es resolen respecte a la carpeta del fitxer.
- **Cartutxos:** fes clic en un cartutx de la graella i defineix-ne el nom, el
  fitxer (**Triar…** o **Treure**), el tipus, **En bucle** i **Aturar els
  altres cartutxos en disparar-lo**.

Les sortides Main i Cue pròpies de la cartutxera són a **Configuració →
Sortides d'àudio** (la fila **Cartutxera**).
