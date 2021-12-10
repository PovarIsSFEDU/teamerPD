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
            _auto_competence.innerHTML += '<li><button class="dropdown-item" id="auto-competence-' + index +'">' + element + '</button></li>'
        });
        recommended_competences.forEach((element, index) => {
            if (!check_autocomplete(element, input)) return
            let htmlelem = document.getElementById('auto-competence-' + index)
            htmlelem.addEventListener("click", () => {_dropdown_method(htmlelem)})
        });
        if (_auto_competence.innerHTML == "") {
            _auto_competence.style.display = "none";
        } else {
            _auto_competence.style.display = "block";
        }
    }
    update_autocomplete()

    function _dropdown_method(e) {
        add_competence(e.textContent)
    }
    
    _input_competence.addEventListener("input", update_autocomplete)


    function _input_method(e) {
        if(e.keyCode == 13) {
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
    
    function remove_competence(i) {
        competences.splice(i, 1)
        update_compentences()
    }
    
    function add_competence(s) {
        if (competences.findIndex((v) => {return v == s}) != -1) return
        competences.push(s)
        update_compentences()
    }

    _input_competence.addEventListener('focus', (event) => {
        _auto_competence.style.display = "block"
        update_autocomplete()
    });

    _input_competence.addEventListener('focusout', (event) => {
        setTimeout(()=>{_auto_competence.style.display = "none";}, 100)
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