-- Your Yazi start-up file. It starts as the system's, from kavat-files, and is yours to
-- change.

-- The status bar says who owns the file under the cursor
Status:children_add(function()
	local hovered = cx.active.current.hovered
	if not hovered or ya.target_family() ~= "unix" then
		return ""
	end
	return ui.Line({
		ui.Span(ya.user_name(hovered.cha.uid) or tostring(hovered.cha.uid)):fg("magenta"),
		":",
		ui.Span(ya.group_name(hovered.cha.gid) or tostring(hovered.cha.gid)):fg("magenta"),
		" ",
	})
end, 500, Status.RIGHT)
