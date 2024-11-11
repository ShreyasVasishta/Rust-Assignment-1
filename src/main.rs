use calamine::{open_workbook_auto,  Reader};
use chrono::{Datelike, NaiveDate, Utc};
use clap::Parser;
use std::fs::File;
use std::io::{BufRead, Write};
use std::collections::HashMap;

#[derive(Parser)]
struct Args {
    #[arg(short = 'e', long = "emp-data-file-path")]
    emp_data_file_path: String,

    #[arg(short = 'd', long = "dept-data-file-path")]
    dept_data_file_path: String,

    #[arg(short = 's', long = "salary-data-file-path")]
    salary_data_file_path: String,

    #[arg(short = 'l', long = "leave-data-file-path")]
    leave_data_file_path: String,

    #[arg(short = 'o', long = "output-file-path")]
    output_file_path: String,
}

struct Employee {
    emp_id: i32,
    emp_name: String,
    dept_id: i32,
    mobile_no: String,
    email: String,
}

 #[derive(Debug)]
struct Department {
    #[allow(dead_code)]
    dept_id: i32,
    dept_title: String,
}

struct Salary {
    emp_id: i32,
    salary_date: String,
    salary_status: String,
}

struct Leave {
    emp_id: i32,
    leave_from: NaiveDate,
    leave_to: NaiveDate,
}

// Load Employee Data
fn load_employee_data(file_path: &str) -> Vec<Employee> {
    let file = File::open(file_path).expect("Could not open employee data file");
    let reader = std::io::BufReader::new(file);
    
    reader.lines().skip(1).filter_map(|line| {
        let line = line.ok()?;
        let parts: Vec<&str> = line.split('|').collect();
        Some(Employee {
            emp_id: parts[0].parse().ok()?,
            emp_name: parts[1].to_string(),
            dept_id: parts[2].parse().ok()?,
            mobile_no: parts[3].to_string(),
            email: parts[4].to_string(),
        })
    }).collect()
}

// Load Department Data
fn load_department_data(file_path: &str) -> HashMap<i32, Department> {
    let mut workbook = open_workbook_auto(file_path).expect("Could not open department file");
    let mut departments = HashMap::new();
    
    if let Some(Ok(range)) = workbook.worksheet_range("Sheet1") {
        for row in range.rows().skip(1) {
            let dept_id: i32 = row[0].get_float().unwrap() as i32;
            let dept_title = row[1].get_string().unwrap().to_string();
            departments.insert(dept_id, Department { dept_id, dept_title });
        }
    }
    departments
}

// fn load_department_data(file_path: &str) -> HashMap<i32, Department> {
//     let mut workbook = open_workbook_auto(file_path).expect("Could not open department file");
//     let mut departments = HashMap::new();

//     // Check if the sheet "Dept" exists and if we can access it
//     if let Some(Ok(range)) = workbook.worksheet_range("Sheet1") {
//         println!("Found 'Dept' sheet, reading rows...");  // Debug print

//         // Loop through rows and read department data
//         for row in range.rows().skip(1) {
//             // Debug print to inspect the row data
//             println!("Row: {:?}", row);

//             let dept_id = row[0].get_float().unwrap_or(0.0) as i32;  // Ensure it's a valid dept_id
//             let dept_title = row[1].get_string().unwrap_or("Unknown").to_string();

//             println!("Adding department: ID = {}, Title = {}", dept_id, dept_title);  // Debug print

//             departments.insert(dept_id, Department { dept_id, dept_title });
//         }
//     } else {
//         println!("'Dept' sheet not found!");  // Debug print
//     }

//     departments
// }

// Load Salary Data
fn load_salary_data(file_path: &str) -> Vec<Salary> {
    let mut workbook = open_workbook_auto(file_path).expect("Could not open salary file");
    let mut salaries = Vec::new();
    
    if let Some(Ok(range)) = workbook.worksheet_range("Sheet1") {
        for row in range.rows().skip(1) {
            let emp_id = row[0].get_float().unwrap() as i32;
            let salary_date = row[2].get_string().unwrap().to_string();
            let salary_status = row[4].get_string().unwrap().to_string();
            
            salaries.push(Salary {
                emp_id,
                salary_date,
                salary_status,
            });
        }
    }
    salaries
}

// Load Leave Data
fn load_leave_data(file_path: &str) -> Vec<Leave> {
    let mut workbook = open_workbook_auto(file_path).expect("Could not open leave data file");
    let mut leaves = Vec::new();

    if let Some(Ok(range)) = workbook.worksheet_range("Sheet1") {
        for row in range.rows().skip(1) {
            let emp_id = row[0].get_float().unwrap_or(0.0) as i32;
            let leave_from = row[2].get_string().unwrap_or("None");
            let leave_to = row[3].get_string().unwrap_or("None");

            let leave_from_parsed = NaiveDate::parse_from_str(leave_from, "%d-%m-%Y").expect("Failed to parse leave_from date");
            let leave_to_parsed = NaiveDate::parse_from_str(leave_to, "%d-%m-%Y").expect("Failed to parse leave_to date");
            println!("{}, {}",leave_from,leave_to);
            println!("{:?},  {:?}",leave_from_parsed,leave_to_parsed);

            leaves.push(Leave {
                emp_id,
                leave_from: leave_from_parsed,
                leave_to: leave_to_parsed,
            });
        }
    }
    leaves
}

// Helper function to get salary status
fn get_salary_status(salaries: &Vec<Salary>, emp_id: i32, current_month: u32, current_year: i32) -> String {
    let current_month_str = format!("{:02}-{}", current_month, current_year);
    for salary in salaries {
        if salary.emp_id == emp_id && salary.salary_date.contains(&current_month_str) && salary.salary_status == "Credited" {
            return "Credited".to_string();
        }
    }
    "Not Credited".to_string()
}

// Helper function to calculate leave days
fn calculate_leave_days(leaves: &Vec<Leave>, emp_id: i32) -> i32 {
    let mut total_days = 0;
    for leave in leaves.iter().filter(|&l| l.emp_id == emp_id) {
        total_days += (leave.leave_to - leave.leave_from).num_days() as i32 + 1;
        // if leave.leave_from.month() == leave.leave_to.month()  {
        //     let end_date = leave.leave_to.min(NaiveDate::from_ymd_opt(current_year, current_month + 1, 1).unwrap().pred_opt().expect("Could not get previous date"));
        //     total_days += (end_date - leave.leave_from).num_days() as i32 + 1;
        // }
    }
    total_days
}

fn main() {
    let args = Args::parse();

    // Load data
    let employees = load_employee_data(&args.emp_data_file_path);
    let departments = load_department_data(&args.dept_data_file_path);
    let salaries = load_salary_data(&args.salary_data_file_path);
    let leaves = load_leave_data(&args.leave_data_file_path);

    //println!("Departments: {:#?}", departments);

    let current_date = Utc::now();
    let current_month = current_date.month();
    let current_year = current_date.year();

    let mut output_file = File::create(&args.output_file_path).expect("Could not create output file");
    writeln!(output_file, "Emp ID~#~Emp Name~#~Dept Title~#~Mobile No~#~Email~#~Salary Status~#~On Leave").unwrap();

    for emp in employees {
        let dept_title = departments
    .get(&emp.dept_id)  // Get the department by emp.dept_id
    .map_or("N/A".to_string(), |dept| dept.dept_title.clone());
        //let dept_title = departments.iter().find(|&dept| dept.dept_id == emp.dept_id).map_or("N/A".to_string(), |dept| dept.dept_title.clone());
        // println!("Employee Dept ID: {}", emp.dept_id);
        // let unknown_title = "Unknown".to_string();  // Store the temporary value
        // let dept_title = departments.get(&emp.dept_id).map(|d| &d.dept_title).unwrap_or(&unknown_title);

        //let dept_title = departments.get(&emp.dept_id).map(|d| &d.dept_title).unwrap_or(&"Unknown".to_string());
        let salary_status = get_salary_status(&salaries, emp.emp_id, current_month, current_year);
        let leave_days = calculate_leave_days(&leaves, emp.emp_id);

        writeln!(
            output_file,
            "{}~#~{}~#~{}~#~{}~#~{}~#~{}~#~{}",
            emp.emp_id,
            emp.emp_name,
            dept_title,
            emp.mobile_no,
            emp.email,
            salary_status,
            leave_days
        ).unwrap();
    }
}
