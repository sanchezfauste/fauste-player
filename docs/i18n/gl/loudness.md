# Normalización de sonoridade

As pistas veñen de moitas fontes e están masterizadas a niveis moi distintos:
un single pop recente pode soar o dobre de forte ca unha gravación dos anos
setenta ou unha promoción falada. A **normalización de sonoridade** reproduce
cada pista e cada cartucho á mesma sonoridade percibida, así que o operador
non ten que mover o fader dunha pista á seguinte.

Está **activada nas instalacións novas**; unha configuración existente conserva
o seu axuste actual (toda configuración gardada xa o garda). Cámbiao en
[Configuración → Sonoridade](settings.md#loudness).

![Configuración, Sonoridade: normalización activada, co obxectivo, o teito de true peak, a ganancia máxima e o axuste máximo nos seus valores predeterminados](../../images/guide/settings-loudness.png)

## Como funciona {#how-it-works}

**Unha ganancia fixa por pista.** Cada pista recibe unha ganancia, en dB,
decidida cando se carga a pista e mantida ata que para. Nada se comprime nin
se limita: toda a pista faise máis forte ou máis baixa na mesma cantidade, así
que a súa propia dinámica queda como foi masterizada. O fader do reprodutor
actúa por riba, como antes.

**A medición.** A análise en segundo plano mide cada ficheiro unha vez:

- a súa **sonoridade integrada**, en LUFS: canto soa de forte o ficheiro
  enteiro, tal como a definen ITU-R BS.1770 e EBU R128 (os pasaxes baixos e o
  silencio quedan fóra por porta, así que un fundido de saída longo ou unha
  pausa non a baixan);
- o seu **true peak**, en dBTP: o nivel máis alto que alcanza o sinal entre
  mostras unha vez convertido a analóxico.

Os ficheiros novos mídense coa súa primeira análise. Os ficheiros engadidos
antes de que se medise a sonoridade mídense nunha **pasada en segundo plano**
que se executa mentres a análise normal non ten nada que facer, sen tocar os
seus marcadores. Execútase estea a normalización activada ou non, así que as
medicións están listas cando a actives. **Volver analizar** (no menú da lista)
mide un ficheiro de novo, lendo o ficheiro aínda que o seu tamaño e data non
cambiasen. Un ficheiro de menos de 0,4 s nunca se mide, e un ficheiro
silencioso non ten sonoridade: ambos se reproducen sen cambios.

**A ganancia.** A ganancia leva a sonoridade medida ao **Obxectivo**, máis o
**Axuste** propio da pista (consulta [Por pista](#per-track-exclude-and-trim)):

> ganancia = obxectivo − sonoridade medida + axuste

Unha pista máis forte có obxectivo baixase tanto como precise. Unha pista máis
baixa sóbese, pero nunca máis da **Ganancia máxima**, e nunca tanto que o seu
true peak superase o **Teito de true peak**: non hai limitador, así que o
teito é o que evita que unha pista subida sature. Unha pista cuxo true peak xa
está por riba do teito non se sobe en absoluto.

Exemplos, cos valores predeterminados (obxectivo −16 LUFS, teito −1 dBTP,
ganancia máxima 12 dB):

| Pista | Medida | True peak | Ganancia |
|---|---|---|---|
| Un master moderno forte | −9.0 LUFS | +0.5 dBTP | −7.0 dB |
| Un master antigo máis baixo | −20.0 LUFS | −10.0 dBTP | +4.0 dB |
| Unha pista baixa con picos altos | −22.0 LUFS | −3.0 dBTP | +2.0 dB (limitada polo teito) |
| Unha gravación moi baixa | −35.0 LUFS | −20.0 dBTP | +12.0 dB (limitada pola ganancia máxima) |

**Cando se decide a ganancia.** Cando se carga unha pista: preparada como
seguinte dun reprodutor, iniciada, cargada en pausa, preescoitada en CUE, ou
disparada como cartucho. Unha pista que se reproduce ou está en pausa
**conserva a súa ganancia** ata que para, a través de buscas e pausas. Cambiar
os axustes, a medición dunha pista ou o seu axuste aplícase a próxima vez que
se carga a pista; a pista seguinte que xa agarda nun reprodutor prepárase de
novo coa nova ganancia, así que comeza con ela.

**Preescoita.** A preescoita CUE e a de cartuchos reproducen unha pista coa
mesma ganancia que recibe en antena, así que o que oes nos auriculares coincide
co programa.

**Vúmetros.** Os vúmetros e a súa lectura de sonoridade miden despois da
ganancia, así que mostran o nivel normalizado. O **Obxectivo de sonoridade**
propio dos vúmetros (Configuración → Vúmetros) só colorea a lectura; non
cambia ningunha ganancia.

## Saídas bit-perfect {#bit-perfect-outputs}

Un reprodutor ou cartucho cuxa saída é [bit-perfect](bit-perfect.md) **nunca se
normaliza**: sempre se reproduce con ganancia unitaria (0 dB), sexan cales
sexan os axustes, porque calquera ganancia cambiaría as mostras que recibe o
dispositivo. Coa normalización activada, a súa cabeceira mostra un distintivo
ámbar **0.0 dB** cuxa dica di «Saída bit-perfect: as pistas reprodúcense sen
alterar.» O mesmo vale para a preescoita nun dispositivo Cue bit-perfect,
mentres a saída Main do reprodutor, se non é bit-perfect, aínda recibe a
ganancia.

Unha pista que xa se está reproducindo (ou está en pausa) cando o seu
dispositivo pasa a ser bit-perfect conserva a ganancia coa que comezou, e o
distintivo **BP** queda apagado para ela. A pista seguinte reprodúcese a 0 dB e
bit-perfect. Unha pista DSD nesa situación reprodúcese convertida a PCM coa
súa ganancia, nunca como un fluxo DSD sen cambios.

## A ganancia na cabeceira do reprodutor {#the-gain-in-the-player-header}

Mentres un reprodutor reproduce ou está en pausa, un distintivo á esquerda de
**BP** mostra a ganancia da pista en antena. O distintivo ten un bordo da súa
cor e sempre o mesmo ancho. Pon o punteiro enriba para ver a razón.

| Distintivo | Significado |
|---|---|
| **+4.0 dB**, **−7.0 dB** (azul) | A pista está normalizada con esta ganancia. A dica é o desglose da ganancia descrito abaixo. Cando a configuración, a medición ou o axuste cambiaron despois de cargar a pista, só di que a pista conserva a ganancia fixada ao cargala, e mostra esa ganancia. |
| **0.0 dB** (ámbar) | A pista reprodúcese sen cambios. A dica é o mesmo desglose, e a súa liña **Ganancia automática** di por que: aínda non ten medición (ou é demasiado curta para medirse), a saída é bit-perfect, ou a pista está excluída da normalización. |
| Sen distintivo | A normalización estaba desactivada cando comezou a pista, a pista é silenciosa, ou o reprodutor está detido. Un reprodutor bit-perfect tampouco mostra ningún mentres a normalización está desactivada. |

O distintivo mostra a ganancia que pediu o reprodutor cando comezou a pista.

### O desglose da ganancia {#the-gain-breakdown}

En todos os lugares nos que o programa explica a ganancia dunha pista mostra o
mesmo bloque de liñas, coa etiqueta á esquerda e o valor á dereita: a dica do
distintivo de ganancia, a xanela emerxente de sonoridade (baixo os seus
controis), o editor de cartuchos e a dica da pista da táboa da lista.

| Liña | Significado |
|---|---|
| **Sonoridade integrada** | A sonoridade medida da pista, en LUFS (ou «sen medir»). |
| **True peak** | O seu true peak máis alto, en dBTP (só cando está medido). |
| **Obxectivo** | A sonoridade á que se leva cada pista (Configuración → Sonoridade). |
| **Ganancia automática** | Obxectivo menos sonoridade medida, antes do axuste e de calquera límite, ou «ningunha» e por que: excluída, saída bit-perfect, normalización desactivada, aínda sen medir, ou silenciosa. |
| **Axuste** | O axuste manual da pista en uso, cando non é 0 (co valor gardado e o máximo cando o limita un **Axuste máximo** reducido). |
| **Ganancia desexada** e **Limitada por** | Só cando o teito de true peak ou a ganancia máxima frearon a ganancia: a ganancia automática máis o axuste (mostrado cando hai axuste) e o límite que a detivo. |
| **Ganancia resultante** | Sempre a última, baixo unha regra e en negriña: a ganancia coa que se reproduce a pista. 0.0 dB cando non está normalizada. |

Coa normalización desactivada, unha nota enriba da ganancia resultante di que
non cambia nada en antena e que o axuste se aplica cando se active.

## Por pista: excluír e axustar {#per-track-exclude-and-trim}

Algunhas pistas non deben seguir o obxectivo: un indicativo da emisora
masterizado forte adrede, un ton de proba, unha pista cuxa medición non
coincide con como soa (a fala adoita soar máis forte ca a música aos mesmos
LUFS). Cada pista ten dous axustes para iso:

- **Excluír da normalización:** a pista sempre se reproduce sen cambios.
- **Axuste:** dB que se suman á ganancia da pista, de − a + o **Axuste máximo**
  (Configuración → Sonoridade), en pasos de 0,5 dB. Fai dobre clic no control
  deslizante para devolvelo a 0. O axuste aparece atenuado mentres a pista
  está excluída.

O control deslizante é un **axuste**: un desprazamento por riba da ganancia
automática, así que le 0 dB para toda pista que non editaches. A xanela
emerxente e o editor de cartuchos mostran por iso o [desglose da
ganancia](#the-gain-breakdown) cos seus controis: a liña **Ganancia
resultante** segue o control mentres o moves. Na xanela emerxente os controis
van primeiro e o desglose debaixo, así que o control deslizante queda onde
está mentres o desglose crece ou encolle. Mentres a pista soa, a xanela
emerxente engade que un cambio se aplica desde a súa próxima carga.

Onde cambialos:

- **Nunha forma de onda:** fai clic dereito na forma de onda dun reprodutor
  (ou na da súa xanela CUE) e escolle **Sonoridade…**. A xanela emerxente
  nomea a pista e mostra a súa medición. Segue editando esa pista aínda que o
  reprodutor pase a outra; péchaa co seu botón **×**. Péchase soa se se quita
  a pista.
- **Nunha fila da lista:** fai clic dereito na fila e escolle
  **Sonoridade…**: a mesma xanela emerxente, para a pista desa fila.
- **No editor de cartuchos:** Configuración → Cartucheira, fai clic nun
  cartucho: os mesmos dous controis están baixo **Parar os outros cartuchos ao
  disparalo**, coa medición baixo a etiqueta.
- **En remoto:** `PUT /api/v1/tracks/<id>/loudness` (consulta
  [Control remoto](remote-control.md)).

Os axustes pertencen á pista e gárdanse con ela. Un cambio aplícase a próxima
vez que se carga a pista, como calquera outro cambio.

**A dica da pista** da táboa da lista mostra as etiquetas da pista e, baixo
unha regra, o mesmo [desglose da ganancia](#the-gain-breakdown), calculado para
o reprodutor ao que pertence esa lista (un reprodutor bit-perfect reprodúceo
sen alterar).

## Escoller os axustes {#choosing-the-settings}

- **Obxectivo.** −16 LUFS convén á maioría dos programas musicais en antena e
  en liña. −23 LUFS (EBU R128) ou −24 LUFS (ATSC A/85) deixan máis marxe, pero
  as pistas baixas precisan entón subidas maiores, que o teito e a ganancia
  máxima poden frear. Escolle un obxectivo próximo a como de forte é xa o teu
  material.
- **Teito de true peak.** −1 dBTP deixa marxe para a conversión e para os
  codificadores con perdas máis adiante. Baixao cando un codificador de
  emisión satura.
- **Ganancia máxima.** Evita que unha gravación moi baixa (ou case toda
  silenciosa) se suba ata converterse en ruído. 0 dB só baixa as pistas fortes.
- **Axuste máximo.** Ata onde pode chegar o axuste por pista. Reducilo limita
  os axustes xa feitos sen cambialos, ata que edites a sonoridade desa pista:
  a edición garda entón o axuste limitado.

## Normas detrás dos valores predeterminados {#standards-behind-the-defaults}

- **Medición.** Sonoridade integrada e true peak tal como os define
  ITU-R BS.1770 (ponderación K, con porta).
- **Obxectivo −16 LUFS.** Segue as recomendacións da AES para streaming e
  distribución en liña. AES TD1008 (2021) substitúe a anterior
  AES TD1004.1.15-10, que recomendaba de −16 a −20 LUFS para streaming. Non é
  un valor de radiodifusión: EBU R128 usa −23 LUFS e ATSC A/85 usa
  −24 LKFS. Usa un deses cando o requira unha especificación de entrega
  (EBU R128 para o intercambio de programas de radiodifusión europeos, ATSC
  A/85 para a televisión dixital dos EUA), e −16 para un programa en
  streaming ou en liña.
- **Teito de true peak −1 dBTP.** O true peak máximo de EBU R128, usado tamén
  polas recomendacións da AES. Os codificadores con perdas poden pasarse ao
  decodificar, así que un fluxo enviado por un codificador de taxa de bits
  baixa pode querer un teito máis baixo, como −2 dBTP (unha práctica común,
  non parte desas normas).
- **Ganancia máxima e axuste máximo, 12 dB.** Decisións de produto sen norma
  detrás: abondo para subir unha gravación baixa e para corrixir unha pista de
  oído, sen tanto que se suba o ruído ou que o axuste anule o obxectivo.
- **A normalización está activada nas instalacións novas.** Actualizar a
  aplicación conserva o axuste que xa tiña a túa configuración, así que unha
  instalación que o tiña desactivado segue desactivada ata que a actives.
- **O obxectivo de sonoridade do vúmetro** (Configuración → Vúmetros) é un
  axuste á parte, por defecto o valor R128 de −23 LUFS. A normalización non o
  usa.
