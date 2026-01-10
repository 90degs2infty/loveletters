#let abs_path(section, page) = {
  let folder(acc, sub_sec) = {
    let current_sec = acc.at(0)
    let current_path = acc.at(1)

    let new_sec = current_sec.subsections.at(sub_sec)
    current_path.push(sub_sec)

    (new_sec, current_path)
  }

  let parent = section.fold((loveletters.project.content, ()), folder)
  let path = parent.at(1)

  if page != none {
    path.push(page)
  }

  let prefix = loveletters.project.config.root.path
  let separator = if prefix.ends-with("/") { "" } else { "/" }
  prefix + separator + path.join("/")
}

#let get_frontmatter() = {
  let folder(current_sec, sub_sec) = {
    current_sec.subsections.at(sub_sec)
  }

  let parent_sec = loveletters.page.path.fold(loveletters.project.content, folder)

  if "page" in loveletters.page.keys() {
    parent_sec.pages.at(loveletters.page.page).frontmatter
  } else {
    parent_sec.index.frontmatter
  }
}
