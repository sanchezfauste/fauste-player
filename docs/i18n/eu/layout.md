# Diseinua

Antolatu leiho nagusia zure txandarako: eman erreproduzitzaileei eta
kartutxo-panelari nahi dituzun tamainak, jarri kartutxo-panela
erreproduzitzaileen azpian edo ondoan, ezkutatu erabiltzen ez dituzun
panelak eta maximizatu bat une batez. Diseinua saioarekin batera gordetzen
da eta hurrengo abiaraztean itzultzen da; instalazio berri batek
erreproduzitzaile guztiak zabalera berean eta kartutxo-panela haien azpian
hasten ditu.

## Erakutsi eta ezkutatu panelak {#show-and-hide-panels}

Goiko barraren ezkerreko lauzek, **Programazioa** eta **Musika**
([programazioa](automation.md#the-schedule-column) eta
[musika-betegarria](automation.md#the-music-filler), biak ezkutuan daude
erakutsi arte), **P1** … **Pn** eta **Kartutxoak**, pantailan zein panel
dauden erakusten dute: piztutako lauza bat erakusten da. Egin klik lauza
batean bere panela ezkutatzeko edo erakusteko. Erreproduzitzaile askorekin,
lauzak alboetara korritzen dira (korritu saguaren gurpila haien gainean) eta
diseinu-botoia ikusgai geratzen da. Beste panelek uzten duen lekua hartzen
dute, eta berriro erakutsitako erreproduzitzaile bat bere zabalerarekin
itzultzen da.

Ezkutatzeak panel bat pantailatik kentzen baino ez du egiten. Ezkutatutako
erreproduzitzaile batek jotzen jarraitzen du, eta bere zenbaki-teklak, MIDIk,
urruneko APIak, bere CUE leihoak eta automatizazioak kontrolatzen jarraitzen
dute. Lauza batek bere panelaren egoera erakusten du, panela erakutsita edo
ezkutatuta egon: puntu gorri bat erreproduzitzaile bat jotzen ari den bitartean
(edo kartutxo-panela kartutxo bat jotzen ari denean) eta bi barra anbar,
Pausatu botoi batek bezala, erreproduzitzaile bat pausan dagoen bitartean.

Erreproduzitzaile bat ezkutatzeak bere hautatutako pista garbitzen du, beraz
**Delete** teklak ezin du ikusten ez duzun pista bat kendu. Leihoan zerrenda
batetik kanpo jaregindako fitxategiak azkenekoz erabili duzun
erreproduzitzaileari edo erakutsitako lehen erreproduzitzaileari doaz;
erreproduzitzaile guztiak ezkutatuta daudenean baztertu egiten dira, eta
egoera-barrak hala dio.

Haien ondoko **Liburutegia** lauzak [liburutegiaren leihoa](library.md) irekitzen
eta ixten du, panelen gainean flotatzen duena; ez da panel bat, beraz azpiko
azken piztutako lauzaren araua ez zaio aplikatzen. Ez da telefono batean
erakusten.

Gutxienez panel bat ikusgai geratzen da: azken lauza piztua ezin da klikatu.
Kartutxo-panela ezkutatuta dagoen bitartean, hura tolesten edo zabaltzen duen
laster-teklak ez du ezer egiten; egin klik **Kartutxoak** lauzan erakusteko.

## Aldatu erreproduzitzaileen tamaina {#resize-the-players}

Arrastatu bi erreproduzitzaileren arteko tartea zabalera bat bestera
mugitzeko. Beste erreproduzitzaileak ez dira mugitzen, eta ezein
erreproduzitzailek ez du bere kontrolek behar dutena baino estuagoa izaten.
Egin klik bikoitza tartean erreproduzitzaile guztiei berriz zabalera bera
emateko.

Leihoa estuegia denean erakutsitako erreproduzitzaile guztientzat beren
txikienean, erreproduzitzaileen errenkada alboetara korritzen da, beti egin
duen bezala.

## Mugitu eta aldatu kartutxo-panelaren tamaina {#move-and-resize-the-cartwall}

Kartutxo-panela erreproduzitzaileen azpian dago, edo leihoaren ezkerrean edo
eskuinean, bere altuera osoarekin: aukeratu **Kartutxo-panela behean**,
**Kartutxo-panela ezkerrean** edo **Kartutxo-panela eskuinean**
[diseinu-menuan](#the-layout-menu). Erreproduzitzaileen azpian, bere
errenkadek behar duten altuerakoa da, goiko barraren eta egoera-barraren
arteko altueraren sei hamarren gehienez; haien ondoan, hasieran zabaleraren
hiru hamarren hartzen ditu.

Kartutxo-panela ez da inoiz bere goiburua (**Gelditu dena**, bere izenburua
eta orri-fitxa bat) edo bere botoi-zutabeak beren txikienean baino estuagoa
izaten. Leihoa estuegia denean hark eta erakutsitako erreproduzitzaile guztiek
beren txikienean sartzeko, erreproduzitzaileen errenkada alboetara korritzen da.

Arrastatu kartutxo-panelaren eta erreproduzitzaileen arteko tartea handiago
edo txikiago egiteko: gora edo behera erreproduzitzaileen azpian dagoenean,
alboetara haien ondoan dagoenean. Erreproduzitzaileek beren kontrolek behar
duten lekua mantentzen dute. Egin klik bikoitza tartean hasierako tamainara
itzultzeko: bere errenkadek behar duten altuera erreproduzitzaileen azpian,
zabaleraren hiru hamarren haien ondoan. Tamaina diseinuarekin gordetzen da;
kartutxo-panela mugitzeak hasierako tamainan hasten du berriro.

Kartutxo-panelaren tamaina aldatu ondoren, edo erreproduzitzaileen ondoan
dagoen bitartean, bere botoiak hazten dira hura betetzeko; izenek eta
denborek beren tamaina mantentzen dute, botoi bakoitzaren erdian. Izen bat
bere hitzen artean doitzen da; bere botoirako luzeegia den hitz bat
puntu-multzo batekin (…) amaitzen da lerroetan zatitu ordez. Kartutxo-panela
bere errenkadek eta zutabeek botoirik txikienetan behar dutena baino
txikiagoa denean, korritu egiten da.

Egin klik **KARTUTXOAK** aukeran kartutxo-panela tolesteko. Erreproduzitzaileen
azpian bere goiburua bakarrik geratzen da; haien ondoan leihoaren altuerako
zerrenda estu bat bihurtzen da, berriro zabaltzeko gezi batekin eta, kartutxo
bat jotzen ari den bitartean, puntu gorri batekin. Zabalduta, bere tamaina
itzultzen du. Tolestuta dagoen bitartean, bere ondoko tarteak ez du ezer
egiten.

## Maximizatu panel bat {#maximize-a-panel}

Erreproduzitzaile bakoitzaren goiburuak, eta kartutxo-panelarenak, bi botoi
txikirekin amaitzen dira: **Ezkutatu** (minus bat) eta **Maximizatu** (kanporantz
begiratzen duten izkinak; **Leheneratu**, barrurantz begiratzen duten izkinak,
maximizatutako panelean). Pasatu sagua izena ikusteko. Ilunduta daude
diseinua blokeatuta dagoenean, **Ezkutatu** ikusgai dagoen azken panelean,
eta **Maximizatu** panel bakar bat ikusgai dagoenean eta tolestutako
kartutxo-panelean. Bi ekintza berberak goiburuaren atzeko planoan (ez bere
botoi batean) egindako eskuineko klikaren menuan daude:

- **Maximizatu**k panel horri leku guztia ematen dio: beste panelak beren
  txikienera txikitzen dira. Une baterako bakarrik da eta ez da gordetzen.
  Menu berean **Leheneratu**k amaitzen du, baita panel bat ezkutatzeak edo
  erakusteak, diseinua blokeatzeak, kartutxo-panela mugitzeak,
  erreproduzitzaile kopurua aldatzeak eta panelen arteko tarte bat arrastatzeak
  edo klik bikoitza egiteak ere; lehengo tamainak itzultzen dira. Tolestutako
  kartutxo-panel bat ezin da maximizatu, eta maximizatutako kartutxo-panel bat
  tolesteak amaitzen du.
- **Ezkutatu**k panela ezkutatzen du, goiko barrako lauzak bezala.

## Diseinu-menua {#the-layout-menu}

Diseinu-botoiak, lauzen eskuinean, menu bat irekitzen du hauekin:

- **Kartutxo-panela behean**, **Kartutxo-panela ezkerrean**,
  **Kartutxo-panela eskuinean**: kartutxo-panela non dagoen;
- **Blokeatu diseinua**: markatuta dagoen bitartean, ezerk ez du diseinua
  nahigabe aldatzen: lauzek, menu honetako beste sarrerek, panelen arteko
  tarteek eta goiburuetako **Maximizatu** eta **Ezkutatu** botoiek ez dute
  ezer egiten. Erreprodukzioak, eta kartutxo-panela tolestu edo zabaltzeak,
  ohi bezala funtzionatzen dute. Blokeoa diseinuarekin gordetzen da;
- **Berdindu erreproduzitzaileen zabalera**: erreproduzitzaile guztiak zabalera
  berean;
- **Berrezarri diseinua**: instalazio berri baten diseinura itzultzen da.
