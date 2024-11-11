use super::*;
use serde_json::json;

#[test]
fn dist() -> Result<(), BuildError> {
    for (name, json) in [
        (
            "countnulls",
            json!({
              "name": "countnulls",
              "releases": {
                 "stable": [
                    {"version": "1.0.0", "date": "2011-03-16T09:36:43Z"}
                 ]
              }
            }),
        ),
        (
            "pair",
            json!({
              "name": "pair",
              "releases": {
                "stable": [
                  {"version": "0.1.7", "date": "2020-10-25T21:54:02Z"},
                  {"version": "0.1.6", "date": "2018-11-10T20:55:55Z"},
                  {"version": "0.1.5", "date": "2011-11-11T17:56:30Z"},
                  {"version": "0.1.4", "date": "2011-11-11T06:52:41Z"},
                  {"version": "0.1.3", "date": "2011-05-12T18:55:30Z"},
                  {"version": "0.1.2", "date": "2011-04-20T23:47:22Z"},
                  {"version": "0.1.1", "date": "2010-10-29T22:44:42Z"},
                  {"version": "0.1.0", "date": "2010-10-19T03:59:54Z"}
                ]
              }
            }),
        ),
        (
            "example",
            json!({
              "name": "example",
              "releases": {
                "stable": [
                  {"version": "1.0.5", "date": "2023-09-10T23:32:07Z"},
                  {"version": "1.0.4", "date": "2020-02-06T18:10:25Z"},
                ],
                "unstable": [
                  {"version": "1.0.5-v1", "date": "2023-09-10T23:32:07Z"},
                  {"version": "1.0.4-v1", "date": "2020-02-06T18:10:25Z"},
                ],
                "testing": [
                  {"version": "1.0.0-b3", "date": "2011-04-22T20:15:25Z"},
                  {"version": "1.0.0-b2", "date": "2011-04-21T22:44:48Z"}
                ]
              }
            }),
        ),
    ] {
        // Write the JSON to a vec, use it as a reader.
        let mut file = Vec::new();
        serde_json::to_writer(&mut file, &json)?;
        let w = file.as_slice();
        let dist = Dist::from_reader(w)?;

        // Check values.
        assert_eq!(
            json.get("name").unwrap().as_str().unwrap(),
            dist.name(),
            "{name} name"
        );

        let releases = json.get("releases").unwrap().as_object().unwrap();
        for (status, list) in [
            ("stable", dist.releases.stable()),
            ("unstable", dist.releases.unstable()),
            ("testing", dist.releases.testing()),
        ] {
            match releases.get(status) {
                None => assert!(list.is_none(), "{name} {status} is none"),
                Some(exp) => {
                    // Make sure the list of releases is the same length.
                    assert!(list.is_some(), "{name} {status} is some");
                    let exp = exp.as_array().unwrap();
                    assert_eq!(exp.len(), list.unwrap().len(), "{name} {status} len");

                    // Make sure the contents are the same.
                    for (i, rel) in exp.iter().enumerate() {
                        let rel = rel.as_object().unwrap();
                        assert_eq!(
                            rel.get("version").unwrap().as_str().unwrap(),
                            list.unwrap().get(i).unwrap().version().to_string(),
                            "{name} {status} {i} version",
                        );
                        assert_eq!(
                            rel.get("date").unwrap().as_str().unwrap(),
                            list.unwrap()
                                .get(i)
                                .unwrap()
                                .date()
                                .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                            "{name} {status} {i} date",
                        );
                    }
                }
            }
        }
    }

    Ok(())
}
