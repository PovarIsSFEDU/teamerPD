var teaminfo = document.getElementById('teaminfo')

teaminfo.addEventListener('show.bs.modal', (event) => {
    var button = event.relatedTarget
    var teamname = button.getAttribute('data-bs-teamname')
    var modalTitle = teaminfo.querySelector('.modal-title')
    var modalBody = teaminfo.querySelector('.modal-body')

    modalTitle.textContent = teamname
    //modalBody.value = recipient
})