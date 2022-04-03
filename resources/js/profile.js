import {sendRequest} from "./sendRequest.js";
import {ThrowError} from "./throwError.js";
import {deleteCookie} from "./deleteCookie.js";

var recommended_competences = []

var competences = []
// Load competences from db

document.addEventListener('DOMContentLoaded', () => {
    const _button_signout = document.querySelector(".signout");
    _button_signout.addEventListener("click", () => {
        sendRequest("GET", "/logout", null)
            .catch(err => ThrowError(err))
        deleteCookie("Authenticate")
        window.location.replace("/login")
    })


    const _buttonSave = document.querySelector("#saveprof");

    let levels = ["nobody", "entry", "intern", "junior", "junior+", "middle", "middle+", "senior"]

    let user_level = document.getElementById("user_lvl")

    for (const level of levels) {
        let button = document.getElementById(level)
        button.addEventListener("click", () => {
            user_level.innerHTML = button.innerHTML + " " + `<i class="uil uil-arrow-down sp-uil"></i>`

        })
    }


    function validate() {
        const _login = document.querySelector("#prof-login")
        if (_login.textContent.toString().replaceAll("/<\\/?.+?>/gi", "") == "") {
            _login.classList.add("invalid")
            return false
        } else {
            _login.classList.remove("invalid")
        }
        const _email = document.querySelector("#prof-email")
        if (_email.textContent.toString().replaceAll("/<\\/?.+?>/gi", "") == "") {
            _email.classList.add("invalid")
            return false
        } else {
            _email.classList.remove("invalid")
        }
        const _firname = document.querySelector("#firname")
        if (_firname.value.toString().trim() == "") {
            _firname.classList.add("invalid")
            return false
        } else {
            _firname.classList.remove("invalid")
        }
        const _surname = document.querySelector("#surname")
        if (_surname.value.toString().trim() == "") {
            _surname.classList.add("invalid")
            return false
        } else {
            _surname.classList.remove("invalid")
        }
        let _comp = document.getElementById("competences")
        if (_comp.innerHTML.trim() == "" || competences.length == 0) {
            document.querySelector("#competence").classList.add("invalid")
            return false
        } else {
            document.querySelector("#competence").classList.remove("invalid")
        }
        return true
    }

    _buttonSave.addEventListener("click", () => {
        if (validate()) {
            let data = {
                login: document.querySelector("#prof-login").textContent,
                name: document.querySelector("#firname").value,
                surname: document.querySelector("#surname").value,
                city: document.querySelector("#city").value,
                tg: document.querySelector("#tg").value,
                git: document.querySelector("#git").value,
                bio: document.querySelector("#bio").value,
                level: document.querySelector("#user_lvl").innerHTML.split(" ")[0],
                team: null,
                photo: null,
                resume: null,
                adm: false,
                email: document.querySelector("#prof-email").textContent,
                competences: competences
            }
            sendRequest("POST", "/api/update_user", data)
                .then(ans => {
                    window.location.reload()
                })
                .catch(err => ThrowError(err))
        }
    })


    const _auto_competence = document.getElementById("competence-auto")
    const _button_competence_add = document.getElementById("competence-add")
    const _div_competences = document.getElementById("competences")
    const _input_competence = document.getElementById("competence")


    function check_autocomplete(cmp, inp) {
        return cmp.toLowerCase().indexOf(inp.toLowerCase()) != -1
    }

    function update_autocomplete() {
        let input = _input_competence.value
        _auto_competence.innerHTML = ""
        recommended_competences.forEach((element, index) => {
            if (!check_autocomplete(element, input)) return
            _auto_competence.innerHTML += '<li><button class="dropdown-item" id="auto-competence-' + index + '">' + element + '</button></li>'
        });
        recommended_competences.forEach((element, index) => {
            if (!check_autocomplete(element, input)) return
            let htmlelem = document.getElementById('auto-competence-' + index)
            htmlelem.addEventListener("click", () => {
                _dropdown_method(htmlelem)
            })
        });
        // if (_auto_competence.innerHTML == "") {
        //     _auto_competence.style.display = "none";
        // } else {
        //     _auto_competence.style.display = "block";
        // }
    }

    update_autocomplete()

    function _dropdown_method(e) {
        add_competence(e.textContent)
    }

    _input_competence.addEventListener("input", update_autocomplete)


    function _input_method(e) {
        if (e.keyCode == 13) {
            _button_method()
        }
    }

    function _button_method() {
        let competence = _input_competence.value
        if (competence == "") return
        _input_competence.value = ""
        add_competence(competence)
    }

    _button_competence_add.addEventListener("click", _button_method)
    _input_competence.addEventListener("keydown", _input_method)

    function update_compentences() {
        _div_competences.innerHTML = ""
        competences.forEach((element, index) => {
            _div_competences.innerHTML += '<h5><span class="badge bg-competence">' + element + '<button class="remove-competence" id="competence-' + index + '"></button></span></h5>'
        });
        competences.forEach((element, index) => {
            let htmlelem = document.getElementById('competence-' + index)
            htmlelem.addEventListener("click", () => {
                remove_competence(index)
            })
        });
    }

    let _rmComp_first = document.querySelectorAll(".remove-competence")
    if (_rmComp_first != null) {
        _rmComp_first.forEach((elem) => {
            competences.push(elem.parentNode.firstChild.textContent)
            elem.addEventListener("click", () => {
                elem.parentNode.parentNode.removeChild(elem.parentNode);
                competences.splice(competences.indexOf(elem.parentNode.firstChild.textContent), 1)
            })
        });
    }

    function remove_competence(i) {
        competences.splice(i, 1)
        update_compentences()
    }

    function add_competence(s) {
        if (competences.findIndex((v) => {
            return v == s
        }) != -1) return
        competences.push(s)
        update_compentences()
    }

    _input_competence.addEventListener('focus', (event) => {
        _auto_competence.style.display = "block"
        update_autocomplete()
    });

    _input_competence.addEventListener('focusout', (event) => {
        setTimeout(() => {
            _auto_competence.style.display = "none";
        }, 100)
    });
});


// Strashno ochen
recommended_competences = [
    "HTML CSS",
    "JavaScript",
    "Python",
    "Django",
    "PHP",
    "Java",
    "Ruby",
    "React",
    "jQuery",
    "Angular",
    "Vue",
    "SQL",
    "R",
    "Go (Golang)",
    "Selenium",
    "C#",
    ".Net",
    "C/C++",
    "Unreal Engine",
    "Unity 3D",
    "Visual Basic",
    "1С",
    "Scala",
    "tensorflow",
    "SOAP",
    "PyTorch",
    "Pascal",
    "Node",
    "NLP",
    "Machine Learning",
    "Linux",
    "Lazarus",
    "Kotlin",
    "keras",
    "Go",
    "Gherkin",
    "Flutter",
    "docker",
    "Delphi",
    "Dart",
    "Cypress",
    "CX-менеджмент",
    "Cucumber",
    "Computer Vision",
    "Blockchain",
    "Технический Писатель",
    "Тестирование",
    "Теория Алгоритмов",
    "Системная Аналитика",
    "Трейд-маркетинг",
    "Мобильный маркетинг",
    "Контекстная реклама",
    "Бренд-стратегия",
    "Управления финансовыми рисками",
    "Управление инвестициями",
    "Финансовое планирование",
    "Юридическая и нормативная база в ИТ",
    "Гибкие методологии управления проектами ",
    "Методы моделирования процессов и программные средства для построения моделей",
    "Project Management",
    "Product Management",
    "Аналитика бизнес-процессов",
    "Figma",
    "Sketch",
    "Adobe Photoshop/ Adobe Illustrator",
    "Инфографика",
    "Проектирование и дизайн веб-интерфейсов",
    "Проектирование и дизайн мобильных интерфейсов",
    "Разработка UX/UI",
    "Мобильная разработка Android",
    "Мобильная разработка iOs",
    "Криптография",
    "Теория защиты информации",
    "Теория Blockchain/распределенный реестр",
    "Теория облачных технологий ",
    "Теория машинного обучения",
    "Теория алгоритмов",
    "Архитектура приложений и базы данных",
    "Основы программирования: типы и структуры данных"
]