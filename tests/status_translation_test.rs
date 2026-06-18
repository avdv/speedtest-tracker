use askama::Template;
use speedtest_tracker::filters;

#[derive(Template)]
#[template(
    source = "{{ status|translate_status(lang) }}",
    ext = "html",
    escape = "none"
)]
struct FmtTemplate<'a> {
    status: &'a str,
    lang: &'a str,
}

#[test]
fn test_translate_all_status_values_en() {
    let cases = [
        ("benchmarking", "Benchmarking"),
        ("checking", "Checking"),
        ("completed", "Completed"),
        ("failed", "Failed"),
        ("running", "Running"),
        ("started", "Started"),
        ("skipped", "Skipped"),
        ("waiting", "Waiting"),
        // Should handle uppercase
        ("COMPLETED", "Completed"),
        ("Failed", "Failed"),
    ];

    for (input, expected) in cases {
        let t = FmtTemplate {
            status: input,
            lang: "en",
        };
        let got = t.render().unwrap();
        assert_eq!(
            got, expected,
            "{} | translate_status({}) returned {:?}",
            input, t.lang, got
        );
    }
}

// #[test]
// fn test_translate_status_german() {
//     rust_i18n::set_locale("de_DE");
// }

// #[test]
// fn test_translate_status_spanish() {
//     rust_i18n::set_locale("es_ES");

//     assert_eq!(translate_status("benchmarking").unwrap(), "Evaluando");
//     assert_eq!(translate_status("checking").unwrap(), "Comprobando");
//     assert_eq!(translate_status("completed").unwrap(), "Completado");
//     assert_eq!(translate_status("failed").unwrap(), "Fallido");
//     assert_eq!(translate_status("running").unwrap(), "Ejecutando");
//     assert_eq!(translate_status("started").unwrap(), "Iniciado");
//     assert_eq!(translate_status("skipped").unwrap(), "Omitido");
//     assert_eq!(translate_status("waiting").unwrap(), "Esperando");
// }

// #[test]
// fn test_translate_status_french() {
//     rust_i18n::set_locale("fr_FR");

//     assert_eq!(
//         translate_status("benchmarking").unwrap(),
//         "Évaluation comparative"
//     );
//     assert_eq!(
//         translate_status("checking").unwrap(),
//         "En cours de vérification"
//     );
//     assert_eq!(translate_status("completed").unwrap(), "Terminé");
//     assert_eq!(translate_status("failed").unwrap(), "Échec");
//     assert_eq!(translate_status("running").unwrap(), "En cours d'exécution");
//     assert_eq!(translate_status("started").unwrap(), "Démarré");
//     assert_eq!(translate_status("skipped").unwrap(), "Ignoré");
//     assert_eq!(translate_status("waiting").unwrap(), "En attente");
// }

// #[test]
// fn test_translate_status_dutch() {
//     rust_i18n::set_locale("nl_NL");

//     assert_eq!(translate_status("benchmarking").unwrap(), "Benchmarking");
//     assert_eq!(translate_status("checking").unwrap(), "Controleren");
//     assert_eq!(translate_status("completed").unwrap(), "Voltooid");
//     assert_eq!(translate_status("failed").unwrap(), "Mislukt");
//     assert_eq!(translate_status("running").unwrap(), "Lopend");
//     assert_eq!(translate_status("started").unwrap(), "Gestart");
//     assert_eq!(translate_status("skipped").unwrap(), "Overgeslagen");
//     assert_eq!(translate_status("waiting").unwrap(), "Wachten");
// }

// #[test]
// fn test_translate_status_portuguese() {
//     rust_i18n::set_locale("pt_BR");

//     assert_eq!(translate_status("benchmarking").unwrap(), "Benchmarking");
//     assert_eq!(translate_status("checking").unwrap(), "Verificando");
//     assert_eq!(translate_status("completed").unwrap(), "Concluído");
//     assert_eq!(translate_status("failed").unwrap(), "Falhou");
//     assert_eq!(translate_status("running").unwrap(), "Executando");
//     assert_eq!(translate_status("started").unwrap(), "Iniciado");
//     assert_eq!(translate_status("skipped").unwrap(), "Ignorado");
//     assert_eq!(translate_status("waiting").unwrap(), "Esperando");
// }
