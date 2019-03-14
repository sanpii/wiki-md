document.addEventListener('DOMContentLoaded', function (){
    var search = document.getElementById('search');

    search.addEventListener('keyup', function (event) {
        var filter = event.target.value.toLowerCase();

        var items = document.getElementsByTagName("li");
        [].forEach.call(items, function (item) {
            if (item.textContent.toLowerCase().indexOf(filter) < 0) {
                item.style.display = 'none';
            }
            else {
                item.style.display = 'list-item';
            }
        });
    });

    search.addEventListener('keydown', function (event) {
        if (event.which === 13) {
            event.preventDefault();
        }
    });
});
