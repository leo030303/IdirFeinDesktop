pub mod page;
mod setup_wizard_utils;
mod update;
mod view;

// Source URL and destination filename
pub const APP_DATA_URLS: [(&str, &str); 190] = [
    (
        "https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/web/styles.css",
        "web/styles.css",
    ),
    (
        "https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/web/template.html",
        "web/template.html",
    ),
    (
        "https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/models/blazefaces-640.onnx",
        "models/blazefaces-640.onnx",
    ),
    (
        "https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/models/face_recognition_sface_2021dec.onnx",
        "models/face_recognition_sface_2021dec.onnx",
    ),
    (
        "https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/models/text-detection.rten",
        "models/text-detection.rten",
    ),
    (
        "https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/models/text-recognition.rten",
        "models/text-recognition.rten",
    ),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/BG/index.aff", "dictionaries/BG/index.aff"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/BG/index.dic", "dictionaries/BG/index.dic"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/BR/index.aff", "dictionaries/BR/index.aff"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/BR/index.dic", "dictionaries/BR/index.dic"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/CA-VALENCIA/index.aff", "dictionaries/CA-VALENCIA/index.aff"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/CA-VALENCIA/index.dic", "dictionaries/CA-VALENCIA/index.dic"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/CA/index.aff", "dictionaries/CA/index.aff"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/CA/index.dic", "dictionaries/CA/index.dic"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/CS/index.aff", "dictionaries/CS/index.aff"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/CS/index.dic", "dictionaries/CS/index.dic"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/CY/index.aff", "dictionaries/CY/index.aff"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/CY/index.dic", "dictionaries/CY/index.dic"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/DA/index.aff", "dictionaries/DA/index.aff"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/DA/index.dic", "dictionaries/DA/index.dic"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/DE-AT/index.aff", "dictionaries/DE-AT/index.aff"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/DE-AT/index.dic", "dictionaries/DE-AT/index.dic"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/DE-CH/index.aff", "dictionaries/DE-CH/index.aff"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/DE-CH/index.dic", "dictionaries/DE-CH/index.dic"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/DE/index.aff", "dictionaries/DE/index.aff"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/DE/index.dic", "dictionaries/DE/index.dic"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/EL-POLYTON/index.aff", "dictionaries/EL-POLYTON/index.aff"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/EL-POLYTON/index.dic", "dictionaries/EL-POLYTON/index.dic"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/EL/index.aff", "dictionaries/EL/index.aff"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/EL/index.dic", "dictionaries/EL/index.dic"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/EN-AU/index.aff", "dictionaries/EN-AU/index.aff"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/EN-AU/index.dic", "dictionaries/EN-AU/index.dic"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/EN-CA/index.aff", "dictionaries/EN-CA/index.aff"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/EN-CA/index.dic", "dictionaries/EN-CA/index.dic"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/EN-GB/index.aff", "dictionaries/EN-GB/index.aff"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/EN-GB/index.dic", "dictionaries/EN-GB/index.dic"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/EN-ZA/index.aff", "dictionaries/EN-ZA/index.aff"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/EN-ZA/index.dic", "dictionaries/EN-ZA/index.dic"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/EN/index.aff", "dictionaries/EN/index.aff"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/EN/index.dic", "dictionaries/EN/index.dic"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/EO/index.aff", "dictionaries/EO/index.aff"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/EO/index.dic", "dictionaries/EO/index.dic"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-AR/index.aff", "dictionaries/ES-AR/index.aff"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-AR/index.dic", "dictionaries/ES-AR/index.dic"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-BO/index.aff", "dictionaries/ES-BO/index.aff"),
    ("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-BO/index.dic", "dictionaries/ES-BO/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-CL/index.aff", "dictionaries/ES-CL/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-CL/index.dic", "dictionaries/ES-CL/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-CO/index.aff", "dictionaries/ES-CO/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-CO/index.dic", "dictionaries/ES-CO/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-CR/index.aff", "dictionaries/ES-CR/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-CR/index.dic", "dictionaries/ES-CR/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-CU/index.aff", "dictionaries/ES-CU/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-CU/index.dic", "dictionaries/ES-CU/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-DO/index.aff", "dictionaries/ES-DO/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-DO/index.dic", "dictionaries/ES-DO/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-EC/index.aff", "dictionaries/ES-EC/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-EC/index.dic", "dictionaries/ES-EC/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-GT/index.aff", "dictionaries/ES-GT/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-GT/index.dic", "dictionaries/ES-GT/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-HN/index.aff", "dictionaries/ES-HN/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-HN/index.dic", "dictionaries/ES-HN/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-MX/index.aff", "dictionaries/ES-MX/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-MX/index.dic", "dictionaries/ES-MX/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-NI/index.aff", "dictionaries/ES-NI/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-NI/index.dic", "dictionaries/ES-NI/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-PA/index.aff", "dictionaries/ES-PA/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-PA/index.dic", "dictionaries/ES-PA/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-PE/index.aff", "dictionaries/ES-PE/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-PE/index.dic", "dictionaries/ES-PE/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-PH/index.aff", "dictionaries/ES-PH/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-PH/index.dic", "dictionaries/ES-PH/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-PR/index.aff", "dictionaries/ES-PR/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-PR/index.dic", "dictionaries/ES-PR/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-PY/index.aff", "dictionaries/ES-PY/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-PY/index.dic", "dictionaries/ES-PY/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-SV/index.aff", "dictionaries/ES-SV/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-SV/index.dic", "dictionaries/ES-SV/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-US/index.aff", "dictionaries/ES-US/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-US/index.dic", "dictionaries/ES-US/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-UY/index.aff", "dictionaries/ES-UY/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-UY/index.dic", "dictionaries/ES-UY/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-VE/index.aff", "dictionaries/ES-VE/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES-VE/index.dic", "dictionaries/ES-VE/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES/index.aff", "dictionaries/ES/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ES/index.dic", "dictionaries/ES/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ET/index.aff", "dictionaries/ET/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/ET/index.dic", "dictionaries/ET/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/EU/index.aff", "dictionaries/EU/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/EU/index.dic", "dictionaries/EU/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/FA/index.aff", "dictionaries/FA/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/FA/index.dic", "dictionaries/FA/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/FO/index.aff", "dictionaries/FO/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/FO/index.dic", "dictionaries/FO/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/FR/index.aff", "dictionaries/FR/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/FR/index.dic", "dictionaries/FR/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/FUR/index.aff", "dictionaries/FUR/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/FUR/index.dic", "dictionaries/FUR/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/FY/index.aff", "dictionaries/FY/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/FY/index.dic", "dictionaries/FY/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/GA/index.aff", "dictionaries/GA/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/GA/index.dic", "dictionaries/GA/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/GD/index.aff", "dictionaries/GD/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/GD/index.dic", "dictionaries/GD/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/GL/index.aff", "dictionaries/GL/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/GL/index.dic", "dictionaries/GL/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/HE/index.aff", "dictionaries/HE/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/HE/index.dic", "dictionaries/HE/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/HR/index.aff", "dictionaries/HR/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/HR/index.dic", "dictionaries/HR/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/HU/index.aff", "dictionaries/HU/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/HU/index.dic", "dictionaries/HU/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/HY/index.aff", "dictionaries/HY/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/HY/index.dic", "dictionaries/HY/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/HYW/index.aff", "dictionaries/HYW/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/HYW/index.dic", "dictionaries/HYW/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/IA/index.aff", "dictionaries/IA/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/IA/index.dic", "dictionaries/IA/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/IE/index.aff", "dictionaries/IE/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/IE/index.dic", "dictionaries/IE/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/IS/index.aff", "dictionaries/IS/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/IS/index.dic", "dictionaries/IS/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/IT/index.aff", "dictionaries/IT/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/IT/index.dic", "dictionaries/IT/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/KA/index.aff", "dictionaries/KA/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/KA/index.dic", "dictionaries/KA/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/KO/index.aff", "dictionaries/KO/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/KO/index.dic", "dictionaries/KO/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/LA/index.aff", "dictionaries/LA/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/LA/index.dic", "dictionaries/LA/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/LB/index.aff", "dictionaries/LB/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/LB/index.dic", "dictionaries/LB/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/LT/index.aff", "dictionaries/LT/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/LT/index.dic", "dictionaries/LT/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/LTG/index.aff", "dictionaries/LTG/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/LTG/index.dic", "dictionaries/LTG/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/LV/index.aff", "dictionaries/LV/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/LV/index.dic", "dictionaries/LV/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/MK/index.aff", "dictionaries/MK/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/MK/index.dic", "dictionaries/MK/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/MN/index.aff", "dictionaries/MN/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/MN/index.dic", "dictionaries/MN/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/NB/index.aff", "dictionaries/NB/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/NB/index.dic", "dictionaries/NB/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/NDS/index.aff", "dictionaries/NDS/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/NDS/index.dic", "dictionaries/NDS/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/NE/index.aff", "dictionaries/NE/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/NE/index.dic", "dictionaries/NE/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/NL/index.aff", "dictionaries/NL/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/NL/index.dic", "dictionaries/NL/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/NN/index.aff", "dictionaries/NN/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/NN/index.dic", "dictionaries/NN/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/OC/index.aff", "dictionaries/OC/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/OC/index.dic", "dictionaries/OC/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/PL/index.aff", "dictionaries/PL/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/PL/index.dic", "dictionaries/PL/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/PT-PT/index.aff", "dictionaries/PT-PT/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/PT-PT/index.dic", "dictionaries/PT-PT/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/PT/index.aff", "dictionaries/PT/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/PT/index.dic", "dictionaries/PT/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/RO/index.aff", "dictionaries/RO/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/RO/index.dic", "dictionaries/RO/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/RU/index.aff", "dictionaries/RU/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/RU/index.dic", "dictionaries/RU/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/RW/index.aff", "dictionaries/RW/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/RW/index.dic", "dictionaries/RW/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/SK/index.aff", "dictionaries/SK/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/SK/index.dic", "dictionaries/SK/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/SL/index.aff", "dictionaries/SL/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/SL/index.dic", "dictionaries/SL/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/SR-LATIN/index.aff", "dictionaries/SR-LATIN/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/SR-LATIN/index.dic", "dictionaries/SR-LATIN/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/SR/index.aff", "dictionaries/SR/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/SR/index.dic", "dictionaries/SR/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/SV-FI/index.aff", "dictionaries/SV-FI/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/SV-FI/index.dic", "dictionaries/SV-FI/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/SV/index.aff", "dictionaries/SV/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/SV/index.dic", "dictionaries/SV/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/TK/index.aff", "dictionaries/TK/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/TK/index.dic", "dictionaries/TK/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/TLH-LATN/index.aff", "dictionaries/TLH-LATN/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/TLH-LATN/index.dic", "dictionaries/TLH-LATN/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/TLH/index.aff", "dictionaries/TLH/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/TLH/index.dic", "dictionaries/TLH/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/TR/index.aff", "dictionaries/TR/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/TR/index.dic", "dictionaries/TR/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/UK/index.aff", "dictionaries/UK/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/UK/index.dic", "dictionaries/UK/index.dic"),

("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/VI/index.aff", "dictionaries/VI/index.aff"),
("https://raw.githubusercontent.com/leo030303/idirfein-resources/refs/heads/main/dictionaries/VI/index.dic", "dictionaries/VI/index.dic"),
];
