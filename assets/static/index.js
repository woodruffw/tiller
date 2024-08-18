const sortListAlpha = list => [...list].sort((a, b) => {
    const A = a.textContent.trim(), B = b.textContent.trim();
    return (A < B) ? -1 : (A > B) ? 1 : 0;
});

const sortListTopicCount = list => [...list].sort((a, b) => {
    const A = parseInt(a.querySelector(".til-tag-count").textContent, 10);
    const B = parseInt(b.querySelector(".til-tag-count").textContent, 10);
    return B - A;
});

function sortAlpha() {
    const ul = document.querySelector(".topic-list");
    const list = ul.querySelectorAll("li");
    ul.append(...sortListAlpha(list));
}

function sortCount() {
    const ul = document.querySelector(".topic-list");
    const list = ul.querySelectorAll("li");
    ul.append(...sortListTopicCount(list));
}
