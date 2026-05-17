/**
 * Generates data/pals.json from Palworld breeding power values (palworld.gg reference).
 * Run: node scripts/build-pal-data.mjs
 */
import { writeFileSync, mkdirSync } from "fs";
import { dirname, join } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");

/** [name, breedingPower] — lower power = rarer */
const PALS = [
  ["Lamball", 1470], ["Cattiva", 1460], ["Chikipi", 1500], ["Lifmunk", 1430],
  ["Foxparks", 1400], ["Fuack", 1330], ["Sparkit", 1410], ["Depresso", 1380],
  ["Cremis", 1455], ["Daedream", 1230], ["Gumoss", 1240], ["Hoocrates", 1390],
  ["Jolthog", 1370], ["Jolthog Cryst", 1360], ["Pengullet", 1350], ["Pengullet Lux", 1310],
  ["Tocotoco", 1340], ["Bristla", 1320], ["Ribbuny", 1310], ["Swee", 1300],
  ["Killamari", 1290], ["Flopie", 1280], ["Kelpsea", 1260], ["Tanzee", 1250],
  ["Woolipop", 1190], ["Dazzi", 1210], ["Fuddler", 1220], ["Cawgnito", 1080],
  ["Maraith", 1150], ["Rooby", 1155], ["Leezpunk", 1120], ["Gobfin", 1090],
  ["Hangyu", 1420], ["Hangyu Cryst", 1422], ["Mau", 1480], ["Mau Cryst", 1440],
  ["Rushoar", 1130], ["Nox", 1180], ["Wixen", 1160], ["Teafant", 1490],
  ["Direhowl", 1060], ["Galeclaw", 1030], ["Gorirat", 1040], ["Vaelet", 1050],
  ["Eikthyrdeer", 920], ["Eikthyrdeer Terra", 900], ["Melpaca", 890], ["Reindrix", 880],
  ["Celaray", 870], ["Mozzarina", 910], ["Lovander", 940], ["Loupmoon", 950],
  ["Fenglope", 980], ["Verdash", 990], ["Robinquill", 1020], ["Felbat", 1010],
  ["Broncherry", 860], ["Caprity", 930], ["Dumud", 895], ["Digtoise", 850],
  ["Tombat", 750], ["Foxcicle", 760], ["Petallia", 780], ["Arsox", 790],
  ["Chillet", 800], ["Kitsun", 830], ["Lullu", 905], ["Croajiro", 795],
  ["Surfent", 560], ["Surfent Terra", 550], ["Azurobe", 500], ["Grintale", 510],
  ["Penking", 520], ["Ghangler", 525], ["Elphidran", 540], ["Sootseer", 545],
  ["Anubis", 570], ["Incineram", 590], ["Incineram Noct", 580], ["Vanwyrm", 660],
  ["Vanwyrm Cryst", 620], ["Bushi", 640], ["Bushi Noct", 650], ["Dogen", 665],
  ["Blazehowl", 710], ["Blazehowl Noct", 670], ["Katress", 700], ["Nitemary", 705],
  ["Rayhound", 740], ["Sibelyx", 450], ["Wumpo", 460], ["Kingpaca", 470],
  ["Azurmane", 400], ["Blazamut", 410], ["Mossanda", 430], ["Palumba", 455],
  ["Quivern", 350], ["Pyrin", 360], ["Ragnahawk", 380], ["Mossanda Lux", 390],
  ["Nitewing", 420], ["Faleris", 370], ["Prixter", 355], ["Warsect", 340],
  ["Elizabee", 330], ["Reptyro", 320], ["Jormuntide", 310], ["Mammorest", 300],
  ["Relaxaurus", 280], ["Knocklem", 265], ["Menasting", 260], ["Lyleen", 250],
  ["Pyrin Noct", 240], ["Reptyro Cryst", 230], ["Beakon", 220], ["Paladius", 220],
  ["Necromus", 200], ["Helzephyr", 190], ["Helzephyr Lux", 180], ["Bastigor", 170],
  ["Cryolinx", 130], ["Cryolinx Terra", 160], ["Frostallion", 120], ["Frostallion Noct", 100],
  ["Jetragon", 90], ["Neptilius", 90], ["Paladius", 80], ["Necromus", 70],
  ["Shadowbeak", 60], ["Suzaku", 50], ["Suzaku Aqua", 30], ["Blazamut Ryu", 9],
  ["Bellanoir", 1], ["Bellanoir Libero", 1],
  ["Grizzbolt", 200], ["Lyleen Noct", 210], ["Selyne", 345], ["Xenolord", 265],
  ["Xenovader", 465], ["Xenogard", 435], ["Silvegis", 215], ["Orserk", 140],
  ["Astegon", 150], ["Menasting Terra", 250], ["Warsect Terra", 275],
  ["Relaxaurus Lux", 270], ["Mammorest Cryst", 290], ["Jormuntide Ignis", 315],
  ["Sweepa", 410], ["Dinossom", 820], ["Dinossom Lux", 810], ["Univolt", 680],
  ["Dazzi Noct", 1115], ["Gorirat Terra", 1030], ["Robinquill Terra", 1000],
  ["Kingpaca Cryst", 440], ["Wumpo Botan", 480], ["Elphidran Aqua", 530],
  ["Azurobe Cryst", 480], ["Broncherry Aqua", 840], ["Caprity Noct", 855],
  ["Kitsun Noct", 735], ["Loupmoon Cryst", 805], ["Fenglope Lux", 835],
  ["Faleris Aqua", 245], ["Quivern Botan", 340], ["Penking Lux", 490],
  ["Chillet Ignis", 790], ["Shroomer", 720], ["Shroomer Noct", 730],
  ["Kikit", 1125], ["Yakumo", 945], ["Mimog", 1200], ["Dazemu", 675],
  ["Omascul", 630], ["Splatterina", 725], ["Tarantriss", 825], ["Gloopie", 1195],
  ["Whalaska", 445], ["Whalaska Ignis", 430], ["Icelyn", 605], ["Herbil", 1445],
  ["Munchill", 1335], ["Finsider", 1295], ["Polapup", 745], ["Braloha", 335],
  ["Frostplume", 655], ["Jellroy", 1395], ["Jelliette", 1385], ["Foxparks Cryst", 1305],
  ["Ribbuny Botan", 1205], ["Flambelle", 1405], ["Smokie", 1245], ["Celesdir", 815],
  ["Starryon", 365], ["Nyafia", 645], ["Prunelia", 755], ["Gildane", 505],
  ["Turtacle", 1105], ["Turtacle Terra", 1065],
];

const SPECIAL_COMBOS = [
  ["Lamball", "Cattiva", "Foxparks"],
  ["Foxparks", "Rooby", "Daedream"],
  ["Blazamut", "Suzaku", "Jormuntide"],
  ["Anubis", "Jetragon", "Frostallion"],
  ["Necromus", "Paladius", "Jetragon"],
  ["Grizzbolt", "Lyleen", "Jetragon"],
  ["Relaxaurus", "Mammorest", "Jetragon"],
  ["Blazamut", "Suzaku Aqua", "Jormuntide Ignis"],
  ["Frostallion", "Frostallion Noct", "Frostallion"],
  ["Penking", "Bushi", "Anubis"],
  ["Incineram", "Maraith", "Incineram Noct"],
  ["Suzaku", "Grizzbolt", "Jetragon"],
];

const dir = join(root, "data");
mkdirSync(dir, { recursive: true });

const pals = PALS.map(([name, power]) => ({ name, power }));
const seen = new Set();
const unique = pals.filter((p) => {
  if (seen.has(p.name)) return false;
  seen.add(p.name);
  return true;
});
unique.sort((a, b) => a.power - b.power);

writeFileSync(
  join(dir, "pals.json"),
  JSON.stringify({ pals: unique }, null, 2),
  "utf8"
);

writeFileSync(
  join(dir, "special_combos.json"),
  JSON.stringify(
    {
      combos: SPECIAL_COMBOS.map(([a, b, child]) => ({ parent_a: a, parent_b: b, child })),
    },
    null,
    2
  ),
  "utf8"
);

console.log(`Wrote ${unique.length} pals and ${SPECIAL_COMBOS.length} special combos.`);
